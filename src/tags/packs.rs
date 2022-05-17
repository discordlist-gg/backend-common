use arc_swap::ArcSwap;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::sync::Arc;

#[cfg(feature = "bincode")]
use bincode::{Decode, Encode};
use once_cell::sync::OnceCell;

use poem_openapi::registry::{MetaSchemaRef, Registry};
use poem_openapi::types::{ParseError, ParseFromJSON, ParseResult, ToJSON, Type};
use scylla::cql_to_rust::{FromCqlVal, FromCqlValError};
use scylla::frame::response::result::CqlValue;
use scylla::frame::value::{Value, ValueTooBig};

use crate::tags::handler::get_tag;
use crate::tags::{Flag, IntoFilter, VisibleTag};

static LOADED_PACK_TAGS: OnceCell<ArcSwap<BTreeMap<String, Flag>>> = OnceCell::new();

pub fn get_pack_tags() -> &'static ArcSwap<BTreeMap<String, Flag>> {
    LOADED_PACK_TAGS.get_or_init(ArcSwap::default)
}

pub fn set_pack_tags(lookup: BTreeMap<String, Flag>) {
    let swap = LOADED_PACK_TAGS.get_or_init(ArcSwap::default);
    swap.store(Arc::new(lookup));
}

#[cfg_attr(feature = "bincode", derive(Encode, Decode))]
#[derive(Default, Clone)]
pub struct PackTags {
    inner: Option<VisibleTag>,
}

impl PackTags {
    pub fn from_raw(tag: String) -> Self {
        let lookup = get_pack_tags();
        let tags = lookup.load();

        if let Some(flag) = get_tag(&tag, tags.as_ref()) {
            Self {
                inner: Some(VisibleTag {
                    name: tag,
                    display_name: flag.display_name.clone(),
                    category: "".to_string(),
                }),
            }
        } else {
            Self::default()
        }
    }

    pub fn as_raw(&self) -> Option<String> {
        self.inner.as_ref().map(|v| v.name.to_string())
    }
}

impl Deref for PackTags {
    type Target = Option<VisibleTag>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Debug for PackTags {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl serde::Serialize for PackTags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde::Serialize::serialize(&self.inner, serializer)
    }
}

impl<'de> serde::Deserialize<'de> for PackTags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let inner: Option<VisibleTag> = Option::deserialize(deserializer)?;
        Ok(Self { inner })
    }
}

impl Type for PackTags {
    const IS_REQUIRED: bool = false;
    type RawValueType = Self;
    type RawElementValueType = <Vec<VisibleTag> as Type>::RawElementValueType;

    fn name() -> Cow<'static, str> {
        Cow::from("Tags<PackTag>")
    }

    fn schema_ref() -> MetaSchemaRef {
        Vec::<String>::schema_ref()
    }

    fn register(registry: &mut Registry) {
        <VisibleTag as Type>::register(registry)
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(self)
    }

    fn raw_element_iter<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &'a Self::RawElementValueType> + 'a> {
        self.inner.raw_element_iter()
    }
}

impl ParseFromJSON for PackTags {
    fn parse_from_json(value: Option<serde_json::Value>) -> ParseResult<Self> {
        if let Some(val) = value {
            let lookup = get_pack_tags();
            let tags = lookup.load();

            let maybe_found = val
                .as_str()
                .and_then(|v| tags.get(&v.to_lowercase()).map(|f| (v, f)));

            let (name, flag) = match maybe_found {
                Some(flag) => flag,
                None => return Err(ParseError::custom(format!("Unknown tag: {}", &val))),
            };

            Ok(Self {
                inner: Some(VisibleTag {
                    name: name.to_lowercase(),
                    display_name: flag.display_name.to_string(),
                    category: flag.category.clone(),
                }),
            })
        } else {
            Err(ParseError::custom("Cannot derive tags from null."))
        }
    }
}

impl ToJSON for PackTags {
    fn to_json(&self) -> Option<serde_json::Value> {
        self.inner.as_ref().map(|v| v.name.clone()).to_json()
    }
}

impl Value for PackTags {
    fn serialize(&self, buf: &mut Vec<u8>) -> Result<(), ValueTooBig> {
        let flags = self.as_raw();
        flags.serialize(buf)?;

        Ok(())
    }
}

impl FromCqlVal<CqlValue> for PackTags {
    fn from_cql(cql_val: CqlValue) -> Result<Self, FromCqlValError> {
        let slf = match cql_val {
            CqlValue::Text(s) => Self::from_raw(s.to_lowercase()),
            _ => Self::default(),
        };

        Ok(slf)
    }
}

impl IntoFilter for PackTags {
    #[inline]
    fn into_filter(self) -> Vec<String> {
        self.inner
            .iter()
            .map(|v| format!("tags = {:?}", v))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lookup() {
        let items = vec![
            (
                "music".into(),
                Flag {
                    display_name: "Music".into(),
                    category: "".to_string(),
                },
            ),
            (
                "moderation".into(),
                Flag {
                    display_name: "Moderation".into(),
                    category: "".to_string(),
                },
            ),
            (
                "utility".into(),
                Flag {
                    display_name: "Utility".into(),
                    category: "".to_string(),
                },
            ),
        ];

        set_pack_tags(BTreeMap::from_iter(items))
    }

    #[test]
    fn test_setting_flags() {
        lookup();

        let sample = serde_json::to_value("music").unwrap();
        let tags =
            PackTags::parse_from_json(Some(sample)).expect("Successful parse from JSON Value.");

        assert_eq!(
            tags.inner,
            Some(VisibleTag {
                name: "music".to_string(),
                display_name: "Music".to_string(),
                category: "".to_string(),
            })
        );
    }

    #[test]
    fn test_loading_flags() {
        lookup();

        let tags = PackTags::from_raw("Moderation-Does-Not_Exist".to_string());

        assert_eq!(tags.inner, None);
    }
}

// #[cfg_attr(feature = "bincode", derive(Encode, Decode))]
// #[derive(
//     Copy,
//     Clone,
//     EnumString,
//     EnumIter,
//     AsRefStr,
//     Display,
//     EnumVariantNames,
//     IntoStaticStr,
//     Debug,
//     serde::Serialize,
//     serde::Deserialize,
//     PartialEq,
//     Eq,
//     Hash,
// )]
// #[strum(serialize_all = "kebab_case", ascii_case_insensitive)]
// #[serde(rename_all = "kebab-case")]
// pub enum PackTags {
//     Games,
//     Utility,
//     Fun,
//     Social,
//     Language,
//     Economy,
//     Moderation,
//     Media,
//     Useful,
//     Educational,
// }
