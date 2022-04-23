use std::borrow::Cow;
use std::str::FromStr;

#[cfg(feature = "bincode")]
use bincode::{Decode, Encode};

use poem_openapi::registry::{MetaSchema, MetaSchemaRef};
use poem_openapi::types::{ParseError, ParseFromJSON, ParseResult, ToJSON, Type};
use scylla::cql_to_rust::{FromCqlVal, FromCqlValError};
use scylla::frame::response::result::CqlValue;
use scylla::frame::value::{Value, ValueTooBig};
use strum::{AsRefStr, Display, EnumIter, EnumString, EnumVariantNames, IntoStaticStr};

use crate::tags::IntoFilter;

#[cfg_attr(feature = "bincode", derive(Encode, Decode))]
#[derive(
    Copy,
    Clone,
    EnumString,
    EnumIter,
    AsRefStr,
    Display,
    EnumVariantNames,
    IntoStaticStr,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
    Hash,
)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive)]
#[serde(rename_all = "kebab-case")]
pub enum BotTags {
    Multilanguage,
    Knowledgebase,
    ReactionRole,
    Math,
    Terraria,
    AutoModeration,
    AnimalCrossing,
    Learn,
    Italian,
    Scripting,
    Roleplay,
    Interactive,
    Stores,
    InviteTracking,
    Weather,
    Counting,
    Valorant,
    NearbyServices,
    Turkish,
    Fortnite,
    French,
    Reddit,
    Reminders,
    TextToSpeach,
    Animation,
    Chinese,
    Playstation,
    Memes,
    Friends,
    Pubg,
    Pokemon,
    Tracking,
    MiniGames,
    Verification,
    Inventory,
    Wikipedia,
    Twitter,
    Chatbot,
    Minecraft,
    Instagram,
    Research,
    Japanese,
    Antispam,
    Nyx,
    Rust,
    Nintendo,
    German,
    Gambling,
    Xbox,
    Games,
    Gta,
    Github,
    RoleManagement,
    Rpg,
    GamingNews,
    Calculator,
    FallGuys,
    Image,
    Russian,
    Twitch,
    Video,
    Documentation,
    Music,
    Webhooks,
    AssassinsCreed,
    TipsTricks,
    Hytale,
    LocalNews,
    Spanish,
    AutoRole,
    Osu,
    Eris,
    Hosting,
    AmongUs,
    Youtube,
    Customcommands,
    Courses,
    Csgo,
    Leaderboards,
    Opensource,
    Profanityfilter,
    Sword,
    Rewards,
    RaidProof,
    Dutch,
    Translation,
    Robbing,
    ApexLegends,
    CustomizableBehavior,
    Romanian,
    English,
    Soundboard,
    Templates,
    Religion,
    Logging,
    Swedish,
    LitCord,
    SeaOfThieves,
    Programming,
    CustomizableFilter,
    Cryptocurrency,
    Gif,
    Norwegian,
    RocketLeague,
    Ticketing,
    Survey,
    Roblox,
    ServerManagement,
    Steam,
    Trivia,
    Anime,
    Meme,
    Game,
    Fun,
    Economy,
    Utility,
    Moderation,
    Leveling,
    League,
    Overwatch,
    Management,
    Media,
    Runescape,
    Web,
    Customizable,
    Social,
    Stream,
    Dashboard,
}

impl Type for BotTags {
    const IS_REQUIRED: bool = false;
    type RawValueType = Self;
    type RawElementValueType = Self;

    fn name() -> Cow<'static, str> {
        Cow::from("BotTag")
    }

    fn schema_ref() -> MetaSchemaRef {
        MetaSchemaRef::Inline(Box::new(MetaSchema::new("BotTags")))
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(self)
    }

    fn raw_element_iter<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &'a Self::RawElementValueType> + 'a> {
        Box::new(vec![self].into_iter())
    }
}

impl ParseFromJSON for BotTags {
    fn parse_from_json(value: Option<serde_json::Value>) -> ParseResult<Self> {
        if let Some(v) = value {
            match v {
                serde_json::Value::String(v) => {
                    Self::from_str(&v).map_err(ParseError::custom)
                },
                _ => Err(ParseError::custom("Invalid tag")),
            }
        } else {
            Err(ParseError::custom("Invalid tag"))
        }
    }
}

impl ToJSON for BotTags {
    fn to_json(&self) -> Option<serde_json::Value> {
        Some(self.to_string().into())
    }
}

impl Value for BotTags {
    fn serialize(&self, buf: &mut Vec<u8>) -> Result<(), ValueTooBig> {
        let s: &str = self.as_ref();
        s.serialize(buf)
    }
}

impl FromCqlVal<CqlValue> for BotTags {
    fn from_cql(cql_val: CqlValue) -> Result<Self, FromCqlValError> {
        let data = match cql_val {
            CqlValue::Text(tag) => {
                Self::from_str(&tag).map_err(|_| FromCqlValError::BadCqlType)?
            },
            _ => return Err(FromCqlValError::BadCqlType),
        };

        Ok(data)
    }
}

impl IntoFilter for Vec<BotTags> {
    #[inline]
    fn into_filter(self) -> Vec<String> {
        self.into_iter()
            .map(|v| format!("tags = {:?}", v.to_string()))
            .collect()
    }
}
