use dioxus::prelude::*;
use dx_preview_macro::preview;

use crate::model::{EnumValue, EnumVariant};

const STYLE: Asset = asset!("./style.css");

#[preview(crate = crate)]
/// A `<select>` dropdown for choosing among a fixed set of named enum variants.
///
/// `value` holds the index (0-based) of the currently selected variant and is
/// updated in place whenever the user picks a different option.
#[component]
pub fn EnumInput(
	/// The static variant list, typically sourced from
	/// [`EnumType::variants`](crate::model::EnumType::variants).
	#[preview(hide)]
	variants: &'static [EnumVariant],
	/// Reactive index of the currently selected variant.
	#[preview(hide)]
	value: Signal<EnumValue>,
) -> Element {
	rsx! {
		document::Link { rel: "stylesheet", href: STYLE }

		select {
			class: "dxp-input-select",
			onchange: move |e| {
				if let Ok(i) = e.value().parse::<u8>() {
					value.set(EnumValue(i));
				}
			},
			for (i, variant) in variants.iter().enumerate() {
				option {
					value: "{i}",
					selected: value().0 as usize == i,
					"{variant.name}"
				}
			}
		}
	}
}
