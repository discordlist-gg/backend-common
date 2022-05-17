use poem_openapi::Object;
use std::collections::BTreeMap;

#[cfg(feature = "bincode")]
use bincode::{Decode, Encode};

#[cfg_attr(feature = "bincode", derive(Encode, Decode))]
#[derive(Debug, Clone, Object, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub struct VisibleTag {
    pub name: String,
    pub display_name: String,
    pub category: String,
}

#[derive(Debug)]
pub struct Flag {
    pub display_name: String,
    pub category: String,
}

pub fn get_tag<'a>(flag: &str, lookup: &'a BTreeMap<String, Flag>) -> Option<&'a Flag> {
    lookup.get(flag)
}

pub fn filter_valid_tags<'a>(
    flags: impl Iterator<Item = &'a String>,
    lookup: &BTreeMap<String, Flag>,
) -> Vec<VisibleTag> {
    let mut named = vec![];
    for name in flags {
        if let Some(flag) = lookup.get(name) {
            named.push(VisibleTag {
                name: name.clone(),
                display_name: flag.display_name.clone(),
                category: flag.category.clone(),
            });
        }
    }

    named
}
