use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;

#[cfg(feature = "bincode")]
use bincode::{Decode, Encode};

use poem_openapi::registry::MetaSchemaRef;
use poem_openapi::types::{ParseError, ParseFromJSON, ParseResult, ToJSON, Type};
use scylla::cql_to_rust::{FromCqlVal, FromCqlValError};
use scylla::frame::response::result::CqlValue;
use scylla::frame::value::ValueTooBig;
use serde::{Deserializer, Serializer};
use serde_json::{json, Value};
use url::Url;

#[cfg_attr(feature = "bincode", derive(Decode, Encode))]
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DiscordUrl(#[cfg_attr(feature = "bincode", bincode(with_serde))] pub Url);

impl serde::Serialize for DiscordUrl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for DiscordUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = Url::deserialize(deserializer)?;
        Ok(Self(inner))
    }
}

impl Default for DiscordUrl {
    fn default() -> Self {
        Self(Url::from_str("https://discordlist.gg/").unwrap())
    }
}

impl Display for DiscordUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for DiscordUrl {
    type Target = Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Type for DiscordUrl {
    const IS_REQUIRED: bool = <Url as Type>::IS_REQUIRED;
    type RawValueType = <Url as Type>::RawValueType;
    type RawElementValueType = <Url as Type>::RawElementValueType;

    fn name() -> Cow<'static, str> {
        Cow::from("DiscordInvite")
    }

    fn schema_ref() -> MetaSchemaRef {
        Url::schema_ref()
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(&self.0)
    }

    fn raw_element_iter<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &'a Self::RawElementValueType> + 'a> {
        self.0.raw_element_iter()
    }
}

impl ToJSON for DiscordUrl {
    fn to_json(&self) -> Option<Value> {
        Some(json!(self.0.to_string()))
    }
}

impl ParseFromJSON for DiscordUrl {
    fn parse_from_json(value: Option<Value>) -> ParseResult<Self> {
        let value = value.ok_or_else(|| ParseError::custom("invalid invite given"))?;

        if let Some(v) = value.as_str() {
            let v = if !v.starts_with("https://") {
                format!("https://{}", v)
            } else {
                v.to_string()
            };

            let url = Url::from_str(&v)?;
            return Ok(Self(url));
        }

        Err(ParseError::custom("invalid invite given"))
    }
}

impl FromStr for DiscordUrl {
    type Err = poem_openapi::types::ParseError<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = Url::from_str(s)?;
        Ok(Self(url))
    }
}

impl FromCqlVal<CqlValue> for DiscordUrl {
    fn from_cql(cql_val: CqlValue) -> Result<Self, FromCqlValError> {
        if let Some(v) = cql_val.as_text() {
            Self::from_str(v).map_err(|_| FromCqlValError::BadCqlType)
        } else {
            Err(FromCqlValError::BadCqlType)
        }
    }
}

impl scylla::frame::value::Value for DiscordUrl {
    fn serialize(&self, buf: &mut Vec<u8>) -> Result<(), ValueTooBig> {
        self.0.as_str().serialize(buf)
    }
}
