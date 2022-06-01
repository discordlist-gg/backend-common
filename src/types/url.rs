use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::ops::Deref;
use std::str::FromStr;

#[cfg(feature = "bincode")]
use bincode::{Decode, Encode};

use poem_openapi::registry::MetaSchemaRef;
use poem_openapi::types::{ParseError, ParseFromJSON, ParseResult, ToJSON, Type};
use scylla::cql_to_rust::{FromCqlVal, FromCqlValError};
use scylla::frame::response::result::CqlValue;
use scylla::frame::value::ValueTooBig;
use serde::{Deserializer, Serializer};
use serde_json::{json, Value};
use url::Url;

#[cfg_attr(feature = "bincode", derive(Decode, Encode))]
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DiscordUrl(#[cfg_attr(feature = "bincode", bincode(with_serde))] pub Url);

impl serde::Serialize for DiscordUrl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for DiscordUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = Url::deserialize(deserializer)?;
        Ok(Self(inner))
    }
}

impl Default for DiscordUrl {
    fn default() -> Self {
        Self(Url::from_str("https://discordlist.gg/").unwrap())
    }
}

impl Display for DiscordUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for DiscordUrl {
    type Target = Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Type for DiscordUrl {
    const IS_REQUIRED: bool = <Url as Type>::IS_REQUIRED;
    type RawValueType = <Url as Type>::RawValueType;
    type RawElementValueType = <Url as Type>::RawElementValueType;

    fn name() -> Cow<'static, str> {
        Cow::from("Url")
    }

    fn schema_ref() -> MetaSchemaRef {
        Url::schema_ref()
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(&self.0)
    }

    fn raw_element_iter<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &'a Self::RawElementValueType> + 'a> {
        self.0.raw_element_iter()
    }
}

impl ToJSON for DiscordUrl {
    fn to_json(&self) -> Option<Value> {
        Some(json!(self.0.to_string()))
    }
}

impl ParseFromJSON for DiscordUrl {
    fn parse_from_json(value: Option<Value>) -> ParseResult<Self> {
        let value = value.ok_or_else(|| ParseError::custom("Invalid url provided."))?;

        if let Some(v) = value.as_str() {
            let url = Url::from_str(v)?;

            if !is_valid_url(&url) {
                return Err(ParseError::custom("Invalid url provided."));
            }

            return Ok(Self(url));
        }

        Err(ParseError::custom("Invalid url provided."))
    }
}

impl FromStr for DiscordUrl {
    type Err = poem_openapi::types::ParseError<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = Url::from_str(s)?;

        if !is_valid_url(&url) {
            return Err(ParseError::custom("Invalid url provided."));
        }

        Ok(Self(url))
    }
}

impl FromCqlVal<CqlValue> for DiscordUrl {
    fn from_cql(cql_val: CqlValue) -> Result<Self, FromCqlValError> {
        if let Some(v) = cql_val.as_text() {
            Self::from_str(v).map_err(|_| FromCqlValError::BadCqlType)
        } else {
            Err(FromCqlValError::BadCqlType)
        }
    }
}

impl scylla::frame::value::Value for DiscordUrl {
    fn serialize(&self, buf: &mut Vec<u8>) -> Result<(), ValueTooBig> {
        self.0.as_str().serialize(buf)
    }
}

fn is_valid_url(url: &Url) -> bool {
    (url.scheme() == "http" || url.scheme() == "https")
        && url.username() == ""
        && url.password().is_none()
        && !url.cannot_be_a_base()
        && url.domain().is_some()
}

#[derive(Clone, Default, PartialEq, Eq, Hash, Debug, serde::Serialize, serde::Deserialize)]
pub struct ConstrainedDiscordUrl<T: constraints::ConstrainedUrl>(pub DiscordUrl, PhantomData<T>);

impl<T: constraints::ConstrainedUrl> From<DiscordUrl> for ConstrainedDiscordUrl<T> {
    fn from(v: DiscordUrl) -> Self {
        Self(v, PhantomData::default())
    }
}

impl<T: constraints::ConstrainedUrl> Display for ConstrainedDiscordUrl<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T: constraints::ConstrainedUrl> Deref for ConstrainedDiscordUrl<T> {
    type Target = DiscordUrl;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: constraints::ConstrainedUrl + Sync + Send + 'static> Type for ConstrainedDiscordUrl<T> {
    const IS_REQUIRED: bool = <DiscordUrl as Type>::IS_REQUIRED;
    type RawValueType = <DiscordUrl as Type>::RawValueType;
    type RawElementValueType = <DiscordUrl as Type>::RawElementValueType;

    fn name() -> Cow<'static, str> {
        DiscordUrl::name()
    }

    fn schema_ref() -> MetaSchemaRef {
        DiscordUrl::schema_ref()
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        self.0.as_raw_value()
    }

    fn raw_element_iter<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &'a Self::RawElementValueType> + 'a> {
        self.0.raw_element_iter()
    }
}

impl<T: constraints::ConstrainedUrl + Sync + Send + 'static> ToJSON for ConstrainedDiscordUrl<T> {
    fn to_json(&self) -> Option<Value> {
        self.0.to_json()
    }
}

impl<T: constraints::ConstrainedUrl + Sync + Send + 'static> ParseFromJSON
    for ConstrainedDiscordUrl<T>
{
    fn parse_from_json(value: Option<Value>) -> ParseResult<Self> {
        let slf = DiscordUrl::parse_from_json(value).map_err(|e| e.propagate())?;

        if !T::is_valid(&slf) {
            Err(ParseError::custom("Invalid url provided."))
        } else {
            Ok(Self::from(slf))
        }
    }
}

impl<T: constraints::ConstrainedUrl + Sync + Send + 'static> FromStr for ConstrainedDiscordUrl<T> {
    type Err = poem_openapi::types::ParseError<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let slf = DiscordUrl::from_str(s).map_err(|e| e.propagate())?;

        if !T::is_valid(&slf) {
            Err(ParseError::custom("Invalid url provided."))
        } else {
            Ok(Self::from(slf))
        }
    }
}

impl<T: constraints::ConstrainedUrl> FromCqlVal<CqlValue> for ConstrainedDiscordUrl<T> {
    fn from_cql(cql_val: CqlValue) -> Result<Self, FromCqlValError> {
        let slf = DiscordUrl::from_cql(cql_val)?;

        if !T::is_valid(&slf) {
            Err(FromCqlValError::BadCqlType)
        } else {
            Ok(Self::from(slf))
        }
    }
}

impl<T: constraints::ConstrainedUrl> scylla::frame::value::Value for ConstrainedDiscordUrl<T> {
    fn serialize(&self, buf: &mut Vec<u8>) -> Result<(), ValueTooBig> {
        self.0.serialize(buf)
    }
}

pub mod constraints {
    use crate::types::DiscordUrl;

    #[inline]
    fn twitter_url(url: &DiscordUrl) -> bool {
        url.domain()
            .map(|v|[
                "twitter.com",
                "www.twitter.com",
            ].contains(&v))
            .unwrap_or_default()
    }

    #[inline]
    fn github_url(url: &DiscordUrl) -> bool {
        url.domain().map(|v| [
            "github.com",
            "gitlab.com",
            "bitbucket.org",
            "www.github.com",
            "www.gitlab.com",
            "www.bitbucket.org",
        ].contains(&v)).unwrap_or_default()
    }

    #[inline]
    fn instagram_url(url: &DiscordUrl) -> bool {
        url.domain()
            .map(|v|[
                "instagram.com",
                "www.instagram.com",
            ].contains(&v))
            .unwrap_or_default()
    }

    pub trait ConstrainedUrl {
        fn is_valid(url: &DiscordUrl) -> bool;
    }

    macro_rules! constraint {
        ($name:ident, $cb:ident) => {
            #[derive(Debug, Copy, Clone)]
            pub struct $name;

            impl $crate::types::url::constraints::ConstrainedUrl for $name {
                fn is_valid(url: &$crate::types::url::DiscordUrl) -> bool {
                    $crate::types::url::constraints::$cb(url)
                }
            }
        };
    }

    constraint!(TwitterUrl, twitter_url);
    constraint!(GitHubUrl, github_url);
    constraint!(InstagramUrl, instagram_url);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::url::constraints::{GitHubUrl, InstagramUrl, TwitterUrl};

    #[test]
    fn test_js_non_http_url() {
        let res = DiscordUrl::from_str("javascript:alert(1)");
        assert!(
            res.is_err(),
            "Expected url rejection for non http(s) protocols."
        );
    }

    #[test]
    fn test_mailto_non_http_url() {
        let res = DiscordUrl::from_str("mailto:test@discordlist.gg");
        assert!(
            res.is_err(),
            "Expected url rejection for non http(s) protocols."
        );
    }

    #[test]
    fn test_github_constrained_url_expect_ok() {
        let res = ConstrainedDiscordUrl::<GitHubUrl>::from_str("https://github.com");
        assert!(res.is_ok(), "Expected url pass for GitHubUrl urls.");
    }

    #[test]
    fn test_github_constrained_url_expect_err() {
        let res = ConstrainedDiscordUrl::<GitHubUrl>::from_str("mailto:test@discordlist.gg");
        assert!(
            res.is_err(),
            "Expected url rejection for non GitHubUrl urls."
        );

        let res = ConstrainedDiscordUrl::<GitHubUrl>::from_str("https://discordlist.gg");
        assert!(
            res.is_err(),
            "Expected url rejection for non GitHubUrl urls."
        );
    }

    #[test]
    fn test_twitter_constrained_url_expect_ok() {
        let res = ConstrainedDiscordUrl::<TwitterUrl>::from_str("https://twitter.com");
        assert!(res.is_ok(), "Expected url pass for TwitterUrl urls.");
    }

    #[test]
    fn test_twitter_constrained_url_expect_err() {
        let res = ConstrainedDiscordUrl::<TwitterUrl>::from_str("mailto:test@discordlist.gg");
        assert!(
            res.is_err(),
            "Expected url rejection for non TwitterUrl urls."
        );

        let res = ConstrainedDiscordUrl::<TwitterUrl>::from_str("https://discordlist.gg");
        assert!(
            res.is_err(),
            "Expected url rejection for non TwitterUrl urls."
        );
    }

    #[test]
    fn test_instagram_constrained_url_expect_ok() {
        let res = ConstrainedDiscordUrl::<InstagramUrl>::from_str("https://instagram.com");
        assert!(res.is_ok(), "Expected url pass for InstagramUrl urls.");
    }

    #[test]
    fn test_instagram_constrained_url_expect_err() {
        let res = ConstrainedDiscordUrl::<InstagramUrl>::from_str("mailto:test@discordlist.gg");
        assert!(
            res.is_err(),
            "Expected url rejection for non InstagramUrl urls."
        );

        let res = ConstrainedDiscordUrl::<InstagramUrl>::from_str("https://discordlist.gg");
        assert!(
            res.is_err(),
            "Expected url rejection for non InstagramUrl urls."
        );
    }
}
