use dioxus::prelude::*;

pub mod input;
pub mod menu;

pub use input::{BoolInput, EnumInput, IntInput, StringInput};
pub use menu::Menu;

/// The shared design-token stylesheet.
///
/// Load this once at the shell level (via [`document::Link`]) and optionally
/// pass it to [`crate::Config::with_css`] so it is also available inside
/// preview iframes.
pub const THEME: Asset = asset!("src/app/ui/theme.css");
