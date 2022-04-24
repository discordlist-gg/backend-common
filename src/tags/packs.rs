use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::sync::Arc;
use arc_swap::ArcSwap;

#[cfg(feature = "bincode")]
use bincode::{Decode, Encode};
use num_bigint::BigInt;
use once_cell::sync::OnceCell;

use poem_openapi::registry::MetaSchemaRef;
use poem_openapi::types::{ParseError, ParseFromJSON, ParseResult, ToJSON, Type};
use scylla::cql_to_rust::{FromCqlVal, FromCqlValError};
use scylla::frame::response::result::CqlValue;
use scylla::frame::value::{Value, ValueTooBig};

use crate::tags::{Flag, from_named_flags, IntoFilter, to_named_flags};

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
    inner: Vec<String>,
}

impl PackTags {
    pub fn from_flags(flags: &BigInt) -> Self {
        let lookup = get_pack_tags();
        let inner = to_named_flags(flags, lookup.load().as_ref());
        Self { inner }
    }

    pub fn to_flags(&self) -> BigInt {
        let lookup = get_pack_tags();
        from_named_flags(&self.inner, lookup.load().as_ref())
    }
}

impl Deref for PackTags {
    type Target = [String];

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
        where S: serde::Serializer
    {
        serde::Serialize::serialize(&self.inner, serializer)
    }
}

impl<'de> serde::Deserialize<'de> for PackTags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de>
    {
        let inner: Vec<String> = Vec::deserialize(deserializer)?;
        Ok(Self {
            inner
        })
    }
}

impl Type for PackTags {
    const IS_REQUIRED: bool = false;
    type RawValueType = Self;
    type RawElementValueType = <Vec<String> as Type>::RawElementValueType;

    fn name() -> Cow<'static, str> {
        Cow::from("Tags<PackTag>")
    }

    fn schema_ref() -> MetaSchemaRef {
        Vec::<String>::schema_ref()
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
            let flag = match val {
                serde_json::Value::String(v) => v,
                other => return Err(ParseError::custom(format!("Cannot derive tags from {:?}", &other))),
            };

            Ok(Self {
                inner: vec![flag]
            })
        } else {
            Err(ParseError::custom("Cannot derive tags from null."))
        }
    }
}

impl ToJSON for PackTags {
    fn to_json(&self) -> Option<serde_json::Value> {
        self.inner.to_json()
    }
}

impl Value for PackTags {
    fn serialize(&self, buf: &mut Vec<u8>) -> Result<(), ValueTooBig> {
        let lookup = get_pack_tags();
        let flags = from_named_flags(&self.inner, lookup.load().as_ref());

        CqlValue::Varint(flags).serialize(buf)?;

        Ok(())
    }
}

impl FromCqlVal<CqlValue> for PackTags {
    fn from_cql(cql_val: CqlValue) -> Result<Self, FromCqlValError> {
        let inst = if let CqlValue::Varint(flags) = cql_val {
            Self::from_flags(&flags)
        } else {
            Self::default()
        };

        Ok(inst)
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
            ("Music".into(), Flag { depreciated: false, flag: 1u64.into() }),
            ("Moderation".into(), Flag { depreciated: false, flag: 2u64.into() }),
            ("Utility".into(), Flag { depreciated: false, flag: 4u64.into() }),
        ];

        set_pack_tags(BTreeMap::from_iter(items))
    }

    #[test]
    fn test_setting_flags() {
        lookup();

        let sample = serde_json::to_value("Music").unwrap();
        let tags = PackTags::parse_from_json(Some(sample)).expect("Successful parse from JSON Value.");

        assert_eq!(tags.inner, vec!["Music"]);
        assert_eq!(tags.to_flags(), 1u64.into());
    }

    #[test]
    fn test_loading_flags() {
        lookup();

        let tags = PackTags::from_flags(&(2u64.into()));

        assert_eq!(tags.inner, vec!["Moderation"]);
        assert_eq!(tags.to_flags(), 2u64.into());
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