mod bots;
mod packs;

pub use bots::BotTags;
pub use packs::PackTags;

pub trait IntoFilter {
    fn into_filter(self) -> Vec<String>;
}
