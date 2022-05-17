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
use serde::de::Error;
use serde::{Deserializer, Serializer};
use serde_json::{json, Value};

use crate::types::PossibleInt;

#[cfg_attr(feature = "bincode", derive(Decode, Encode))]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct JsSafeInt(pub i32);

impl serde::Serialize for JsSafeInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for JsSafeInt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = PossibleInt::deserialize(deserializer)?;
        let slf = match inner {
            PossibleInt::Int(v) => Self(v as i32),
            PossibleInt::Str(v) => Self(v.parse::<i32>().map_err(D::Error::custom)?),
        };

        Ok(slf)
    }
}

impl Display for JsSafeInt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for JsSafeInt {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Type for JsSafeInt {
    const IS_REQUIRED: bool = <i32 as Type>::IS_REQUIRED;
    type RawValueType = <i32 as Type>::RawValueType;
    type RawElementValueType = <i32 as Type>::RawElementValueType;

    fn name() -> Cow<'static, str> {
        Cow::from("Integer")
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

impl ToJSON for JsSafeInt {
    fn to_json(&self) -> Option<Value> {
        Some(json!(self.0))
    }
}

impl ParseFromJSON for JsSafeInt {
    fn parse_from_json(value: Option<Value>) -> ParseResult<Self> {
        let v = value.ok_or_else(|| ParseError::custom("cannot convert value into integer"))?;

        let slf = match v {
            Value::String(v) => Self::from_str(&v)?,
            other => other
                .as_i64()
                .map(|v| v as i32)
                .map(Self)
                .ok_or_else(|| ParseError::custom("cannot convert value into integer"))?,
        };

        Ok(slf)
    }
}

impl FromStr for JsSafeInt {
    type Err = poem_openapi::types::ParseError<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = s.parse::<i32>()?;
        Ok(Self(id))
    }
}

impl FromCqlVal<CqlValue> for JsSafeInt {
    fn from_cql(cql_val: CqlValue) -> Result<Self, FromCqlValError> {
        cql_val
            .as_int()
            .map(|v| Self(v))
            .ok_or(FromCqlValError::BadCqlType)
    }
}

impl scylla::frame::value::Value for JsSafeInt {
    fn serialize(&self, buf: &mut Vec<u8>) -> Result<(), ValueTooBig> {
        self.0.serialize(buf)
    }
}
