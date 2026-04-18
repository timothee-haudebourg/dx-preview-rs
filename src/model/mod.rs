//! Data model shared between the shell and the isolation page.
//!
//! The central type is [`ComponentEntry`], which is registered at link time by
//! the [`preview`](crate::preview) attribute macro and collected via
//! [`inventory`]. The shell iterates over all collected entries to populate the
//! sidebar and property editor.

pub use inventory;

mod property;
mod r#type;
mod value;

pub use property::*;
pub use r#type::*;
pub use value::*;

/// A component registered at compile time by the [`preview`](crate::preview) attribute.
///
/// Each annotated component contributes one `ComponentEntry` to the
/// [`inventory`] registry. The shell reads this registry on startup to
/// populate the sidebar.
pub struct ComponentEntry {
	/// Name of the component.
	pub name: &'static str,

	/// Component properties.
	pub properties: &'static [Property],

	/// Default value for each property.
	pub default_values: fn() -> Vec<Option<Value>>,

	/// Render the component with the given properties.
	pub render: fn(Vec<Option<Value>>) -> dioxus::prelude::Element,
}

impl PartialEq for ComponentEntry {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name
	}
}

inventory::collect!(ComponentEntry);
