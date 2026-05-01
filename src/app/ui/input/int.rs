use dioxus::prelude::*;
use dx_preview_macro::preview;

const STYLE: Asset = asset!("./style.css");

#[preview(crate = crate, layout = crate::app::PaddedLayout)]
/// A numeric input for integer properties.
///
/// Reads and writes directly to the provided [`Signal<i64>`].
#[component]
pub fn IntInput(value: Signal<i64>) -> Element {
	rsx! {
		document::Link { rel: "stylesheet", href: STYLE }

		input {
			r#type: "number",
			value: "{value}",
			class: "dxp-input-number",
			oninput: move |e| {
			    if let Ok(n) = e.value().parse::<i64>() {
			        value.set(n);
			    }
			},
		}
	}
}
