use std::{fmt::Display, str::FromStr};

use dioxus::prelude::*;

const STYLE: Asset = asset!("./style.css");

/// A numeric input for integer properties.
///
/// Reads and writes directly to the provided [`Signal<i64>`].
#[component]
pub fn IntInput<V>(value: Signal<V>) -> Element
where
	V: 'static + FromStr + Display,
{
	rsx! {
		document::Link { rel: "stylesheet", href: STYLE }

		input {
			r#type: "number",
			value: "{value}",
			class: "dxp-input-number",
			oninput: move |e| {
			if let Ok(n) = e.value().parse::<V>() {
				value.set(n);
			}
			},
		}
	}
}

#[cfg(feature = "preview")]
mod preview {
	use dioxus::prelude::*;
	use dx_preview_macro::preview;

	#[preview(crate = crate)]
	#[component]
	pub fn IntInput(value: Signal<i64>) -> Element {
		super::IntInput(super::IntInputProps { value })
	}
}
