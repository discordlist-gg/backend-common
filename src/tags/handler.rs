use std::collections::BTreeMap;
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
    pub category: String,
}

pub fn is_valid_tag(flag: &str, lookup: &BTreeMap<String, Flag>) -> bool {
    lookup.contains_key(flag)
}

pub fn filter_valid_tags<'a>(flags: impl Iterator<Item = &'a String>, lookup: &BTreeMap<String, Flag>) -> Vec<VisibleTag> {
    let mut named = vec![];
    for name in flags {
        if let Some(flag) = lookup.get(name) {
            named.push(VisibleTag { name: name.to_string(), category: flag.category.clone() });
        }
    }

    named
}
