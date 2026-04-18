use dioxus::prelude::*;
use dx_preview_macro::preview;

#[preview(crate = crate)]
/// A checkbox input bound to a [`Signal<bool>`].
#[component]
pub fn BoolInput(#[preview(hide)] value: Signal<bool>) -> Element {
	rsx! {
		label {
			style: "display:flex;align-items:center;gap:8px;cursor:pointer;",
			input {
				r#type: "checkbox",
				checked: value(),
				style: "width:16px;height:16px;cursor:pointer;accent-color:#2563eb;",
				onchange: move |e| value.set(e.checked()),
			}
			span {
				style: "font-size:13px;color:#111827;",
				if value() { "true" } else { "false" }
			}
		}
	}
}
