mod bots;
mod handler;
mod packs;

pub use bots::{get_bot_tags, set_bot_tags, BotTags};
pub use handler::{filter_valid_tags, Flag, VisibleTag};
pub use packs::{get_pack_tags, set_pack_tags, PackTags};

pub trait IntoFilter {
    fn into_filter(self) -> Vec<String>;
}
