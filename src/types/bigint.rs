use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use std::ops::Deref;
use std::str::FromStr;

#[cfg(feature = "bincode")]
use bincode::{Decode, Encode};

use poem_openapi::registry::MetaSchemaRef;
use poem_openapi::types::{ParseError, ParseFromJSON, ParseResult, ToJSON, Type};
use scylla::cql_to_rust::{FromCqlVal, FromCqlValError};
use scylla::frame::response::result::CqlValue;
use scylla::frame::value::ValueTooBig;
use serde::de::Error;
use serde::{Deserializer, Serializer};
use serde_json::{json, Value};

use crate::types::PossibleInt;

#[cfg_attr(feature = "bincode", derive(Decode, Encode))]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct JsSafeBigInt(pub i64);

impl serde::Serialize for JsSafeBigInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.to_string().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for JsSafeBigInt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = PossibleInt::deserialize(deserializer)?;
        let slf = match inner {
            PossibleInt::Int(v) => Self(v),
            PossibleInt::Str(v) => Self(v.parse::<i64>().map_err(D::Error::custom)?),
        };

        Ok(slf)
    }
}

impl From<i64> for JsSafeBigInt {
    fn from(v: i64) -> Self {
        Self(v)
    }
}

impl Display for JsSafeBigInt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for JsSafeBigInt {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Type for JsSafeBigInt {
    const IS_REQUIRED: bool = <String as Type>::IS_REQUIRED;
    type RawValueType = <i64 as Type>::RawValueType;
    type RawElementValueType = <i64 as Type>::RawElementValueType;

    fn name() -> Cow<'static, str> {
        Cow::from("BigInt")
    }

    fn schema_ref() -> MetaSchemaRef {
        String::schema_ref()
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

impl ToJSON for JsSafeBigInt {
    fn to_json(&self) -> Option<Value> {
        Some(json!(self.0.to_string()))
    }
}

impl ParseFromJSON for JsSafeBigInt {
    fn parse_from_json(value: Option<Value>) -> ParseResult<Self> {
        let v = value.ok_or_else(|| ParseError::custom("cannot convert value into integer"))?;

        let slf = match v {
            Value::String(v) => Self::from_str(&v)?,
            other => other
                .as_i64()
                .map(Self)
                .ok_or_else(|| ParseError::custom("cannot convert value into integer"))?,
        };

        Ok(slf)
    }
}

impl FromStr for JsSafeBigInt {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = s.parse::<i64>()?;
        Ok(Self(id))
    }
}

impl FromCqlVal<CqlValue> for JsSafeBigInt {
    fn from_cql(cql_val: CqlValue) -> Result<Self, FromCqlValError> {
        match cql_val {
            CqlValue::Counter(c) => Ok(Self(c.0)),
            CqlValue::BigInt(v) => Ok(Self(v)),
            _ => Err(FromCqlValError::BadCqlType),
        }
    }
}

impl scylla::frame::value::Value for JsSafeBigInt {
    fn serialize(&self, buf: &mut Vec<u8>) -> Result<(), ValueTooBig> {
        scylla::frame::value::Value::serialize(&self.0, buf)
    }
}
