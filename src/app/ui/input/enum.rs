use dioxus::prelude::*;
use dx_preview_macro::preview;

use crate::model::EnumVariant;

#[preview(crate = crate)]
/// A `<select>` dropdown for choosing among a fixed set of named enum variants.
///
/// `signal` holds the index (0-based) of the currently selected variant and is
/// updated in place whenever the user picks a different option.
#[component]
pub fn EnumInput(
	/// The static variant list, typically sourced from
	/// [`EnumType::variants`](crate::model::EnumType::variants).
	#[preview(hide)]
	variants: &'static [EnumVariant],
	/// Reactive index of the currently selected variant.
	#[preview(hide)]
	value: Signal<u8>,
) -> Element {
	rsx! {
		select {
			style: "width:100%;box-sizing:border-box;border:1px solid #d1d5db;border-radius:4px;padding:5px 8px;font-size:13px;outline:none;background:#fff;cursor:pointer;",
			onchange: move |e| {
				if let Ok(i) = e.value().parse::<u8>() {
					value.set(i);
				}
			},
			for (i, variant) in variants.iter().enumerate() {
				option {
					value: "{i}",
					selected: value() as usize == i,
					"{variant.name}"
				}
			}
		}
	}
}
