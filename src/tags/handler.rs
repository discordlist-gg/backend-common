use std::borrow::Borrow;
use std::collections::BTreeMap;
use num_bigint::BigInt;


pub struct Flag {
    pub depreciated: bool,
    pub flag: BigInt,
}


pub fn to_named_flags(flags: &BigInt, lookup: &BTreeMap<String, Flag>) -> Vec<String> {
    let zero = BigInt::from(0u8);
    let mut named = vec![];
    for (key, value) in lookup {
        if !value.depreciated && (flags & value.flag.borrow()) != zero {
            named.push(key.to_string());
        }
    }

    named
}

pub fn from_named_flags<'a>(
    named: impl IntoIterator<Item = &'a String>,
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