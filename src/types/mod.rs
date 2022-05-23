mod bigint;
mod integer;
mod invite;
mod set;
mod timestamp;
mod unicode_aware;
mod url;

pub use bigint::JsSafeBigInt;
pub use integer::JsSafeInt;
pub use invite::DiscordInvite;
pub use set::Set;
pub use timestamp::Timestamp;
pub use unicode_aware::NormalisingString;
pub use self::url::DiscordUrl;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum PossibleInt {
    Int(i64),
    Str(String),
}
