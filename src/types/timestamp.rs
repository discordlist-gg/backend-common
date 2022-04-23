use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;

#[cfg(feature = "bincode")]
use bincode::{
    Decode,
    Encode,
    de::Decoder,
    enc::Encoder,
    error::{DecodeError, EncodeError},
};

use chrono::{NaiveDateTime, Utc};
use poem_openapi::registry::MetaSchemaRef;
use poem_openapi::types::{ParseError, ParseFromJSON, ParseResult, ToJSON, Type};
use scylla::cql_to_rust::{FromCqlVal, FromCqlValError};
use scylla::frame::response::result::CqlValue;
use scylla::frame::value::ValueTooBig;
use serde::de::Error;
use serde::{Deserializer, Serializer};
use serde_json::{json, Value};

use crate::types::PossibleInt;

type DateTime = chrono::DateTime<chrono::Utc>;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Timestamp(pub DateTime);

impl serde::Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.to_rfc3339().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = PossibleInt::deserialize(deserializer)?;
        let slf = match inner {
            PossibleInt::Int(v) => Self::from(v),
            PossibleInt::Str(v) => Self::from_str(&v)
                .map_err(|_| D::Error::custom("Cannot convert string to timestamp."))?,
        };

        Ok(slf)
    }
}

#[cfg(feature = "bincode")]
impl Encode for Timestamp {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.0.timestamp().encode(encoder)
    }
}

#[cfg(feature = "bincode")]
impl Decode for Timestamp {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let inner = i64::decode(decoder)?;
        Ok(Self::from(inner))
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self(Utc::now())
    }
}

impl From<i64> for Timestamp {
    fn from(v: i64) -> Self {
        Self(DateTime::from_utc(
            NaiveDateTime::from_timestamp_opt(v, 0).unwrap(),
            Utc,
        ))
    }
}

impl Display for Timestamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for Timestamp {
    type Target = DateTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Type for Timestamp {
    const IS_REQUIRED: bool = <DateTime as Type>::IS_REQUIRED;
    type RawValueType = <DateTime as Type>::RawValueType;
    type RawElementValueType = <DateTime as Type>::RawElementValueType;

    fn name() -> Cow<'static, str> {
        Cow::from("Timestamp")
    }

    fn schema_ref() -> MetaSchemaRef {
        i64::schema_ref()
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

impl ToJSON for Timestamp {
    fn to_json(&self) -> Option<Value> {
        Some(json!(self.0.to_rfc3339()))
    }
}

impl ParseFromJSON for Timestamp {
    fn parse_from_json(value: Option<Value>) -> ParseResult<Self> {
        let value =
            value.ok_or_else(|| ParseError::custom("invalid timestamp given"))?;

        if let Some(v) = value.as_i64() {
            return Ok(Self::from(v));
        }

        if let Some(v) = value.as_str() {
            return Self::from_str(v);
        }

        Err(ParseError::custom("invalid timestamp given"))
    }
}

impl FromStr for Timestamp {
    type Err = poem_openapi::types::ParseError<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = DateTime::from_str(s)?;
        Ok(Self(id))
    }
}

impl FromCqlVal<CqlValue> for Timestamp {
    fn from_cql(cql_val: CqlValue) -> Result<Self, FromCqlValError> {
        cql_val
            .as_duration()
            .map(|v| Self::from(v.num_seconds()))
            .ok_or(FromCqlValError::BadCqlType)
    }
}

impl scylla::frame::value::Value for Timestamp {
    fn serialize(&self, buf: &mut Vec<u8>) -> Result<(), ValueTooBig> {
        self.0.timestamp_millis().serialize(buf)
    }
}
