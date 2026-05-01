use dioxus::prelude::*;
use dx_preview_macro::preview;

const STYLE: Asset = asset!("./style.css");

#[preview(crate = crate, layout = crate::app::PaddedLayout)]
/// A checkbox input bound to a [`Signal<bool>`].
#[component]
pub fn BoolInput(value: Signal<bool>) -> Element {
	rsx! {
		document::Link { rel: "stylesheet", href: STYLE }

		label { class: "dxp-input-bool",
			input {
				r#type: "checkbox",
				checked: value(),
				onchange: move |e| value.set(e.checked()),
			}
			span {
				if value() {
					"true"
				} else {
					"false"
				}
			}
		}
	}
}
