use dioxus::prelude::*;

mod panel;

use panel::*;

use crate::{
	app::{
		Route,
		protocol::{CHILD_ID, use_parent},
	},
	model::ComponentEntry,
};

#[component]
pub fn ComponentView(entry: &'static ComponentEntry) -> Element {
	let iframe_ready = use_parent();

	let values = entry
		.properties
		.iter()
		.map(|prop| use_signal(|| (prop.default_value)()))
		.collect::<Vec<_>>();

	let src = use_memo({
		let values = values.clone();
		move || {
			let values: Vec<_> = values.iter().map(|v| v()).collect();
			let json = serde_json::to_string(&values).unwrap_or_default();

			Route::ComponentPage {
				name: entry.name.to_string(),
				props: json,
			}
			.to_string()
		}
	});

	rsx! {
		div {
			class: "component-view",

			iframe {
				id: CHILD_ID,
				src: "{src()}",
				class: if iframe_ready() { "ready" } else { "loading" },
			}

			Panel { entry, iframe_ready, values }
		}
	}
}
