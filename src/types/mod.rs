mod bigint;
mod integer;
mod url;
mod set;
mod timestamp;
mod unicode_aware;
mod invite;

pub use bigint::JsSafeBigInt;
pub use integer::JsSafeInt;
pub use url::DiscordUrl;
pub use invite::DiscordInvite;
pub use set::Set;
pub use timestamp::Timestamp;
pub use unicode_aware::NormalisingString;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum PossibleInt {
    Int(i64),
    Str(String),
}
