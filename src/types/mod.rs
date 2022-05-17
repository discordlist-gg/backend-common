mod bigint;
mod integer;
mod invite;
mod maybe;
mod set;
mod timestamp;

pub use bigint::JsSafeBigInt;
pub use integer::JsSafeInt;
pub use invite::DiscordUrl;
pub use maybe::MaybeMissing;
pub use set::Set;
pub use timestamp::Timestamp;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum PossibleInt {
    Int(i64),
    Str(String),
}
