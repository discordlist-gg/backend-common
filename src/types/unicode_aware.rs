use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use serde::{Deserializer, Serializer};

#[cfg(feature = "bincode")]
use bincode::{
    de::Decoder,
    enc::Encoder,
    error::{DecodeError, EncodeError},
    Decode, Encode,
};
use poem_openapi::registry::MetaSchemaRef;
use poem_openapi::types::{ParseError, ParseFromJSON, ParseResult, ToJSON, Type};
use scylla::cql_to_rust::{FromCqlVal, FromCqlValError};
use scylla::frame::response::result::CqlValue;
use scylla::frame::value::ValueTooBig;
use serde_json::Value;


#[cfg_attr(feature = "bincode", derive(Decode, Encode))]
pub struct NormalisingString<const MIN: usize, const MAX: usize> {
    normalised: String,
    real: String,
}

impl<const MIN: usize, const MAX: usize> From<&str> for NormalisingString<MIN, MAX> {
    fn from(v: &str) -> Self {
        let normalised = deunicode::deunicode(v);
        Self {
            normalised: normalised.trim().to_string(),
            real: v.trim().to_string(),
        }
    }
}

impl<const MIN: usize, const MAX: usize> From<String> for NormalisingString<MIN, MAX> {
    fn from(real: String) -> Self {
        Self::from(real.as_str())
    }
}

impl<const MIN: usize, const MAX: usize> Display for NormalisingString<MIN, MAX> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.normalised)
    }
}

impl<const MIN: usize, const MAX: usize> Debug for NormalisingString<MIN, MAX> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Normalised({:?})", &self.normalised)
    }
}

impl<const MIN: usize, const MAX: usize> Deref for NormalisingString<MIN, MAX> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.real.as_str()
    }
}

impl<const MIN: usize, const MAX: usize> NormalisingString<MIN, MAX> {
    #[inline]
    pub fn as_raw(&self) -> &str {
        self.real.as_str()
    }

    #[inline]
    pub fn as_normalized(&self) -> &str {
        self.normalised.as_str()
    }
}

impl<const MIN: usize, const MAX: usize> serde::Serialize for NormalisingString<MIN, MAX> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.real.serialize(serializer)
    }
}

impl<'de, const MIN: usize, const MAX: usize> serde::Deserialize<'de> for NormalisingString<MIN, MAX> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let real = String::deserialize(deserializer)?;
        Ok(Self::from(real))
    }
}


impl<const MIN: usize, const MAX: usize> Type for NormalisingString<MIN, MAX> {
    const IS_REQUIRED: bool = <String as Type>::IS_REQUIRED;
    type RawValueType = Self;
    type RawElementValueType = Self;

    fn name() -> Cow<'static, str> {
        String::name()
    }

    fn schema_ref() -> MetaSchemaRef {
        String::schema_ref()
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(&self)
    }

    fn raw_element_iter<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &'a Self::RawElementValueType> + 'a> {
        Box::new(vec![self].into_iter())
    }
}

impl<const MIN: usize, const MAX: usize> ToJSON for NormalisingString<MIN, MAX> {
    fn to_json(&self) -> Option<Value> {
        Some(Value::String(self.real.clone()))
    }
}

impl<const MIN: usize, const MAX: usize> ParseFromJSON for NormalisingString<MIN, MAX> {
    fn parse_from_json(value: Option<Value>) -> ParseResult<Self> {
        let value = value
            .ok_or_else(|| ParseError::custom("Expected type 'String' got null"))?
            .to_string();

        let slf = Self::from(value);

        if slf.normalised.len() < MIN {
            return Err(ParseError::custom(format!("Normalised string value is bellow the minimum length threshold of {} characters.", MIN)))
        }

        if slf.normalised.len() > MAX {
            return Err(ParseError::custom(format!("Normalised string value is above the maximum length threshold of {} characters.", MAX)))
        }

        if slf.real.len() < MIN {
            return Err(ParseError::custom(format!("Raw string value is bellow the minimum length threshold of {} characters.", MIN)))
        }

        if slf.real.len() > MAX {
            return Err(ParseError::custom(format!("Raw string value is above the maximum length threshold of {} characters.", MAX)))
        }

        Ok(slf)
    }
}

impl<const MIN: usize, const MAX: usize> FromCqlVal<CqlValue> for NormalisingString<MIN, MAX> {
    fn from_cql(cql_val: CqlValue) -> Result<Self, FromCqlValError> {
        let s = String::from_cql(cql_val)?;
        Ok(Self::from(s))
    }
}

impl<const MIN: usize, const MAX: usize> scylla::frame::value::Value for NormalisingString<MIN, MAX> {
    fn serialize(&self, buf: &mut Vec<u8>) -> Result<(), ValueTooBig> {
        self.real.serialize(buf)
    }
}



#[cfg(test)]
mod tests {
    use serde_json::json;
    use super::*;

    #[allow(clippy::invisible_characters)]
    #[test]
    fn test_raw_length_handling() {
        let thing = "​​​​​ hi ​​";

        let s = NormalisingString::<5, 20>::parse_from_json(Some(json!(thing)));
        assert!(s.is_err(), "Expected length validation to fail");
    }

    #[allow(clippy::invisible_characters)]
    #[test]
    fn test_normalised_length_handling() {
        let thing = "hi ​";

        let s = NormalisingString::<8, 20>::parse_from_json(Some(json!(thing)));
        assert!(s.is_err(), "Expected length validation to fail");
    }

    #[allow(clippy::invisible_characters)]
    #[test]
    fn test_no_unicode() {
        let thing = "hi ";

        let s = NormalisingString::<5, 20>::parse_from_json(Some(json!(thing)));
        assert!(s.is_ok(), "Expected successful parse");
    }
}