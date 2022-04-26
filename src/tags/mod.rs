mod bots;
mod packs;
mod handler;

pub use handler::{from_named_flags, to_named_flags, Flag, VisibleTag};
pub use bots::{BotTags, set_bot_tags, get_bot_tags};
pub use packs::{PackTags, set_pack_tags, get_pack_tags};

pub trait IntoFilter {
    fn into_filter(self) -> Vec<String>;
}
