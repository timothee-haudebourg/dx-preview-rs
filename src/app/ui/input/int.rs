use dioxus::prelude::*;
use dx_preview_macro::preview;

#[preview(crate = crate)]
/// A numeric input for integer properties.
///
/// Reads and writes directly to the provided [`Signal<i64>`].
#[component]
pub fn IntInput(#[preview(hide)] value: Signal<i64>) -> Element {
	rsx! {
		input {
			r#type: "number",
			value: "{value}",
			style: "width:100%;box-sizing:border-box;border:1px solid #d1d5db;border-radius:4px;padding:5px 8px;font-size:13px;outline:none;background:#fff;",
			oninput: move |e| {
				if let Ok(n) = e.value().parse::<i64>() {
					value.set(n);
				}
			},
		}
	}
}
