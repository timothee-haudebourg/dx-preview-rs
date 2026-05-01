use dioxus::prelude::*;

mod button;
mod dropdown;
pub mod input;
pub mod menu;
mod popup;

pub use button::*;
pub use dropdown::*;
use dx_preview_macro::Reflect;
pub use input::{BoolInput, EnumInput, IntInput, StringInput};
pub use menu::Menu;
pub use popup::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect)]
#[preview(crate = crate)]
pub enum Role {
	#[default]
	Normal,
	Destructive,
}

/// The shared design-token stylesheet.
///
/// Load this once at the shell level (via [`document::Link`]) and optionally
/// pass it to [`crate::Config::with_css`] so it is also available inside
/// preview iframes.
pub const THEME: Asset = asset!("src/app/ui/theme.css");
