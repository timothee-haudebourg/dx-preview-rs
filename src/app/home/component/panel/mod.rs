use dioxus::prelude::*;

mod property;

use property::*;

use crate::model::{ComponentEntry, Value};

const STYLE: Asset = asset!("./style.css");

#[component]
pub fn Panel(
	entry: &'static ComponentEntry,
	iframe_ready: Signal<bool>,
	values: Vec<Signal<Value>>,
) -> Element {
	if entry.properties.is_empty() {
		return rsx! {
			div { class: "dxp-properties-panel", "data-empty": "true", "No editable properties" }
		};
	}

	rsx! {
		div { class: "dxp-properties-panel",

			document::Link { rel: "stylesheet", href: STYLE }

			header { "Properties" }

			div {
				for (prop, value) in entry.properties.iter().zip(values) {
					PropertyEditor { prop, value }
				}
			}
		}
	}
}
