use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::slice::SliceIndex;

#[cfg(feature = "bincode")]
use bincode::{Decode, Encode};

use poem_openapi::registry::{MetaSchemaRef, Registry};
use poem_openapi::types::{ParseError, ParseFromJSON, ParseResult, ToJSON, Type};
use scylla::cql_to_rust::{FromCqlVal, FromCqlValError};
use scylla::frame::response::result::CqlValue;
use scylla::frame::value::ValueTooBig;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg_attr(feature = "bincode", derive(Decode, Encode))]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Set<T>(pub Vec<T>);

impl<T> Set<T> {
    #[inline]
    pub fn push(&mut self, v: T) {
        self.0.push(v)
    }

}

impl<T: PartialEq> Set<T> {
    pub fn insert_no_dupe(&mut self, v: T) {
        if self.0.contains(&v) {
            return;
        }

        self.0.push(v);
    }

    #[inline]
    pub fn contains(&self, v: &T) -> bool {
        self.0.contains(v)
    }
}

impl<T> Default for Set<T> {
    fn default() -> Self {
        Self(vec![])
    }
}

impl<T> From<Vec<T>> for Set<T> {
    fn from(v: Vec<T>) -> Self {
        Self(v)
    }
}

impl<T> FromIterator<T> for Set<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(Vec::from_iter(iter))
    }
}

impl<T> Display for Set<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Set<{}>", std::any::type_name::<T>())
    }
}

impl<T> Deref for Set<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Set<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> AsMut<Vec<T>> for Set<T> {
    fn as_mut(&mut self) -> &mut Vec<T> {
        &mut self.0
    }
}

impl<T: Type> Type for Set<T> {
    const IS_REQUIRED: bool = true;
    type RawValueType = <Vec<T> as Type>::RawValueType;
    type RawElementValueType = <Vec<T> as Type>::RawElementValueType;

    fn name() -> Cow<'static, str> {
        Cow::Owned(format!("Set<{}>", T::name()))
    }

    fn schema_ref() -> MetaSchemaRef {
        Vec::<T>::schema_ref()
    }

    fn register(registry: &mut Registry) {
        T::register(registry);
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        self.0.as_raw_value()
    }

    fn raw_element_iter<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &'a Self::RawElementValueType> + 'a> {
        self.0.raw_element_iter()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<T: ToJSON> ToJSON for Set<T> {
    fn to_json(&self) -> Option<Value> {
        self.0.to_json()
    }
}

impl<T: ParseFromJSON> ParseFromJSON for Set<T> {
    fn parse_from_json(value: Option<Value>) -> ParseResult<Self> {
        let inner =
            Vec::<T>::parse_from_json(value).map_err(|e| ParseError::custom(e.into_message()))?;

        Ok(Self(inner))
    }
}

impl<T: FromCqlVal<CqlValue>> FromCqlVal<Option<CqlValue>> for Set<T> {
    fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
        if let Some(v) = cql_val {
            Ok(Self(Vec::<T>::from_cql(v)?))
        } else {
            Ok(Self(Default::default()))
        }
    }
}

impl<T: scylla::frame::value::Value> scylla::frame::value::Value for Set<T> {
    fn serialize(&self, buf: &mut Vec<u8>) -> Result<(), ValueTooBig> {
        self.0.serialize(buf)
    }
}
