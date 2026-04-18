use std::cell::RefCell;

use dioxus::prelude::Asset;

/// Configuration for the storybook.
///
/// # Example
/// ```rust
/// const TAILWIND: Asset = asset!("/assets/tailwind.css");
///
/// fn main() {
///     dx_preview::launch(
///         dx_preview::Config::default()
///             .with_css(TAILWIND),
///     );
/// }
/// ```
#[derive(Default)]
pub struct Config {
	css: Vec<Asset>,
}

impl Config {
	pub fn new() -> Self {
		Self::default()
	}

	/// Inject a CSS asset into every story iframe.
	pub fn with_css(mut self, asset: Asset) -> Self {
		self.css.push(asset);
		self
	}

	pub(crate) fn apply(self) {
		CONFIG.with(|c| *c.borrow_mut() = Some(self));
	}
}

thread_local! {
	static CONFIG: RefCell<Option<Config>> = const { RefCell::new(None) };
}

pub fn css_assets() -> Vec<Asset> {
	CONFIG.with(|c| {
		c.borrow()
			.as_ref()
			.map(|cfg| cfg.css.clone())
			.unwrap_or_default()
	})
}
