use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::sync::Arc;
use arc_swap::ArcSwap;

#[cfg(feature = "bincode")]
use bincode::{Decode, Encode};
use once_cell::sync::OnceCell;
use inflector::Inflector;

use poem_openapi::registry::{MetaSchemaRef, Registry};
use poem_openapi::types::{ParseError, ParseFromJSON, ParseResult, ToJSON, Type};
use scylla::cql_to_rust::{FromCqlVal, FromCqlValError};
use scylla::frame::response::result::CqlValue;
use scylla::frame::value::{Value, ValueTooBig};

use crate::tags::{Flag, IntoFilter, filter_valid_tags, VisibleTag};


static LOADED_BOT_TAGS: OnceCell<ArcSwap<BTreeMap<String, Flag>>> = OnceCell::new();

pub fn get_bot_tags() -> &'static ArcSwap<BTreeMap<String, Flag>> {
    LOADED_BOT_TAGS.get_or_init(ArcSwap::default)
}

pub fn set_bot_tags(lookup: BTreeMap<String, Flag>) {
    let swap = LOADED_BOT_TAGS.get_or_init(ArcSwap::default);
    swap.store(Arc::new(lookup));
}


#[cfg_attr(feature = "bincode", derive(Encode, Decode))]
#[derive(Default, Clone)]
pub struct BotTags {
    inner: Vec<VisibleTag>,
}

impl BotTags {
    pub fn from_raw(flags: &[String]) -> Self {
        let lookup = get_bot_tags();
        let inner = filter_valid_tags(flags.iter(), lookup.load().as_ref());
        Self { inner }
    }

    pub fn as_raw(&self) -> Vec<String> {
        self.inner
            .iter()
            .map(|v| v.name.to_string())
            .collect()
    }
}

impl Deref for BotTags {
    type Target = [VisibleTag];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Debug for BotTags {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl serde::Serialize for BotTags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
        serde::Serialize::serialize(&self.inner, serializer)
    }
}

impl<'de> serde::Deserialize<'de> for BotTags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de>
    {
        let inner: Vec<VisibleTag> = Vec::deserialize(deserializer)?;
        Ok(Self {
            inner
        })
    }
}

impl Type for BotTags {
    const IS_REQUIRED: bool = false;
    type RawValueType = Self;
    type RawElementValueType = <Vec<VisibleTag> as Type>::RawElementValueType;

    fn name() -> Cow<'static, str> {
        Cow::from("Tags<BotTag>")
    }

    fn schema_ref() -> MetaSchemaRef {
        Vec::<String>::schema_ref()
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(self)
    }

    fn register(registry: &mut Registry) {
        <VisibleTag as Type>::register(registry)
    }

    fn raw_element_iter<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &'a Self::RawElementValueType> + 'a> {
        self.inner.raw_element_iter()
    }
}

impl ParseFromJSON for BotTags {
    fn parse_from_json(value: Option<serde_json::Value>) -> ParseResult<Self> {
        if let Some(val) = value {
            let flags: Vec<String> = match serde_json::from_value(val) {
                Ok(flags) => flags,
                Err(e) => return Err(ParseError::custom(format!("Cannot derive tags: {}", e))),
            };

            let lookup = get_bot_tags();
            let tags = lookup.load();

            let mut inner = vec![];
            for flag_name in flags {
                let flag_name = flag_name.to_title_case();
                let flag = match tags.get(&flag_name) {
                    Some(v) => v,
                    None => return Err(ParseError::custom(format!("Unknown tag: {:?}", flag_name)))
                };

                let visible = VisibleTag { name: flag_name, category: flag.category.clone() };
                inner.push(visible)
            }

            Ok(Self {
                inner
            })
        } else {
            Err(ParseError::custom("Cannot derive tags from null."))
        }
    }
}

impl ToJSON for BotTags {
    fn to_json(&self) -> Option<serde_json::Value> {
        self.inner
            .iter()
            .map(|v| v.name.clone())
            .collect::<Vec<_>>()
            .to_json()
    }
}

impl Value for BotTags {
    fn serialize(&self, buf: &mut Vec<u8>) -> Result<(), ValueTooBig> {
        let flags = self.as_raw();
        flags.serialize(buf)?;

        Ok(())
    }
}

impl FromCqlVal<CqlValue> for BotTags {
    fn from_cql(cql_val: CqlValue) -> Result<Self, FromCqlValError> {
        let values = match cql_val {
            CqlValue::Set(items) => items,
            _ => return Ok(Self::default())
        };

        let iter = values
            .iter()
            .filter_map(|v| v.as_text());

        let lookup = get_bot_tags();
        let inner = filter_valid_tags(iter, lookup.load().as_ref());

        Ok(Self {
            inner
        })
    }
}

impl IntoFilter for BotTags {
    #[inline]
    fn into_filter(self) -> Vec<String> {
        self.inner
            .iter()
            .map(|v| format!("tags = {:?}", v))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lookup() {
        let items = vec![
            ("Music".into(), Flag { category: "".to_string() }),
            ("Moderation".into(), Flag { category: "".to_string() }),
            ("Utility".into(), Flag { category: "".to_string() }),
        ];

        set_bot_tags(BTreeMap::from_iter(items))
    }

    #[test]
    fn test_setting_flags() {
        lookup();


        let sample = serde_json::to_value(vec!["Music", "Hello", "Utility"]).unwrap();
        assert!(BotTags::parse_from_json(Some(sample)).is_err());

        let sample = serde_json::to_value(vec!["Music", "Utility"]).unwrap();
        let tags = BotTags::parse_from_json(Some(sample)).expect("Successful parse from JSON Value.");

        assert_eq!(
            tags.inner,
            vec![
                VisibleTag { name: "Music".to_string(), category: "".to_string() },
                VisibleTag { name: "Utility".to_string(), category: "".to_string() },
            ],
        );
    }

    #[test]
    fn test_loading_flags() {
        lookup();

        let sample = vec![
            "Music".into(),
            "Moderation".into(),
            "Utility".into(),
            "Cheese".into(),
        ];

        let tags = BotTags::from_raw(&sample);

        assert_eq!(
            tags.inner,
            vec![
                VisibleTag { name: "Music".to_string(), category: "".to_string() },
                VisibleTag { name: "Moderation".to_string(), category: "".to_string() },
                VisibleTag { name: "Utility".to_string(), category: "".to_string() },
            ],
        );
    }
}

// #[cfg_attr(feature = "bincode", derive(Encode, Decode))]
// #[derive(
//     Copy,
//     Clone,
//     EnumString,
//     EnumIter,
//     AsRefStr,
//     Display,
//     EnumVariantNames,
//     IntoStaticStr,
//     Debug,
//     serde::Serialize,
//     serde::Deserialize,
//     PartialEq,
//     Eq,
//     Hash,
// )]
// #[strum(serialize_all = "kebab_case", ascii_case_insensitive)]
// #[serde(rename_all = "kebab-case")]
// pub enum BotTags {
//     MultiLanguage,
//     KnowledgeBase,
//     ReactionRole,
//     Math,
//     Terraria,
//     AutoModeration,
//     AnimalCrossing,
//     Learn,
//     Italian,
//     Scripting,
//     Roleplay,
//     Interactive,
//     Stores,
//     InviteTracking,
//     Weather,
//     Counting,
//     Valorant,
//     NearbyServices,
//     Turkish,
//     Fortnite,
//     French,
//     Reddit,
//     Reminders,
//     TextToSpeech,
//     Animation,
//     Chinese,
//     Playstation,
//     Memes,
//     Friends,
//     Pubg,
//     Pokemon,
//     Tracking,
//     MiniGames,
//     Verification,
//     Inventory,
//     Wikipedia,
//     Twitter,
//     ChatBot,
//     Minecraft,
//     Instagram,
//     Research,
//     Japanese,
//     Antispam,
//     Nyx,
//     Rust,
//     Nintendo,
//     German,
//     Gambling,
//     Xbox,
//     Games,
//     Gta,
//     Github,
//     RoleManagement,
//     Rpg,
//     GamingNews,
//     Calculator,
//     FallGuys,
//     Image,
//     Russian,
//     Twitch,
//     Video,
//     Documentation,
//     Music,
//     Webhooks,
//     AssassinsCreed,
//     TipsTricks,
//     Hytale,
//     LocalNews,
//     Spanish,
//     AutoRole,
//
//     #[strum(serialize = "osu!")]
//     #[serde(rename = "osu!")]
//     Osu,
//     Eris,
//     Hosting,
//     AmongUs,
//     Youtube,
//     CustomCommands,
//     Courses,
//     Csgo,
//     Leaderboards,
//     Opensource,
//     ProfanityFilter,
//     Sword,
//     Rewards,
//     RaidProof,
//     Dutch,
//     Translation,
//     Robbing,
//     ApexLegends,
//     CustomizableBehavior,
//     Romanian,
//     English,
//     Soundboard,
//     Templates,
//     Religion,
//     Logging,
//     Swedish,
//     LitCord,
//     SeaOfThieves,
//     Programming,
//     CustomizableFilter,
//     Cryptocurrency,
//     Gif,
//     Norwegian,
//     RocketLeague,
//     Ticketing,
//     Survey,
//     Roblox,
//     ServerManagement,
//     Steam,
//     Trivia,
//     Anime,
//     Meme,
//     Game,
//     Fun,
//     Economy,
//     Utility,
//     Moderation,
//     Leveling,
//     League,
//     Overwatch,
//     Management,
//     Media,
//     Runescape,
//     Web,
//     Customizable,
//     Social,
//     Stream,
//     Dashboard,
// }
//
//
// impl IntoFilter for Vec<BotTags> {
//     #[inline]
//     fn into_filter(self) -> Vec<String> {
//         self.into_iter()
//             .map(|v| format!("tags = {:?}", v.to_string()))
//             .collect()
//     }
// }
