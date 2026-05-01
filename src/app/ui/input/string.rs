use dioxus::prelude::*;
use dx_preview_macro::preview;

const STYLE: Asset = asset!("./style.css");

#[preview(crate = crate, layout = crate::app::PaddedLayout)]
/// A text input bound to a [`Signal<String>`].
#[component]
pub fn StringInput(value: Signal<String>) -> Element {
	rsx! {
		document::Link { rel: "stylesheet", href: STYLE }

		input {
			r#type: "text",
			class: "dxp-input-text",
			value: "{value}",
			oninput: move |e| value.set(e.value()),
		}
	}
}
