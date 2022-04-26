use std::borrow::Borrow;
use std::collections::BTreeMap;
use num_bigint::BigInt;
use poem_openapi::Object;

#[cfg(feature = "bincode")]
use bincode::{Decode, Encode};

#[cfg_attr(feature = "bincode", derive(Encode, Decode))]
#[derive(Debug, Clone, Object, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub struct VisibleTag {
    pub name: String,
    pub category: String,
}

#[derive(Debug)]
pub struct Flag {
    pub flag: BigInt,
    pub category: String,
    pub depreciated: bool,
}

pub fn to_named_flags(flags: &BigInt, lookup: &BTreeMap<String, Flag>) -> Vec<VisibleTag> {
    let zero = BigInt::from(0u8);
    let mut named = vec![];
    for (key, value) in lookup {
        if !value.depreciated && (flags & value.flag.borrow()) != zero {
            named.push(VisibleTag { name: key.to_string(), category: value.category.clone() });
        }
    }

    named
}

pub fn from_named_flags<'a>(
    named: impl Iterator<Item = &'a String>,
    lookup: &BTreeMap<String, Flag>,
) -> BigInt {
    let mut flags = BigInt::from(0u8);
    for named_flag in named {
        if let Some(val) = lookup.get(named_flag) {
            if !val.depreciated {
                flags |= val.flag.borrow();
            }
        }
    }

    flags
}