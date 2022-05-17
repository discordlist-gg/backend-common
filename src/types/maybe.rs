use poem_openapi::registry::MetaSchemaRef;
use poem_openapi::types::{ParseError, ParseFromJSON, ParseResult, ToJSON, Type};
use serde_json::Value;
use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};

pub enum MaybeMissing<T> {
    Provided(T),
    Missing,
}

impl<T> Default for MaybeMissing<T> {
    fn default() -> Self {
        Self::Missing
    }
}

impl<T: Debug> Debug for MaybeMissing<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing => write!(f, "Missing"),
            Self::Provided(v) => write!(f, "Provided({:?})", v),
        }
    }
}

impl<T: Type> Type for MaybeMissing<T> {
    const IS_REQUIRED: bool = false;
    type RawValueType = T;
    type RawElementValueType = T;

    fn name() -> Cow<'static, str> {
        T::name()
    }

    fn schema_ref() -> MetaSchemaRef {
        T::schema_ref()
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        match self {
            Self::Missing => None,
            Self::Provided(v) => Some(v),
        }
    }

    fn raw_element_iter<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &'a Self::RawElementValueType> + 'a> {
        match self {
            Self::Missing => Box::new(vec![].into_iter()),
            Self::Provided(v) => Box::new(vec![v].into_iter()),
        }
    }
}

impl<T: ToJSON> ToJSON for MaybeMissing<T> {
    fn to_json(&self) -> Option<Value> {
        match self {
            Self::Missing => None,
            Self::Provided(v) => v.to_json(),
        }
    }
}

impl<T: ParseFromJSON> ParseFromJSON for MaybeMissing<T> {
    fn parse_from_json(value: Option<Value>) -> ParseResult<Self> {
        match value {
            None => Ok(Self::Missing),
            Some(v) => Ok(Self::Provided(
                T::parse_from_json(Some(v)).map_err(ParseError::propagate)?,
            )),
        }
    }
}
