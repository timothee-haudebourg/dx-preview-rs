use dioxus::prelude::*;
use dx_preview_macro::preview;

#[preview(crate = crate)]
/// A text input for a string property.
#[component]
pub fn StringInput(value: Signal<String>) -> Element {
	rsx! {
		input {
			r#type: "text",
			value: "{value}",
			style: "width:100%;box-sizing:border-box;border:1px solid #d1d5db;border-radius:4px;padding:5px 8px;font-size:13px;outline:none;background:#fff;",
			oninput: move |e| value.set(e.value()),
		}
	}
}
