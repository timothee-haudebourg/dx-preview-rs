use dioxus::prelude::*;

mod panel;

use panel::*;

use crate::{
	app::{
		Route,
		protocol::{CHILD_ID, set_child_value, use_parent},
	},
	model::ComponentEntry,
};

#[component]
pub fn ComponentView(entry: &'static ComponentEntry) -> Element {
	let iframe_ready = use_parent();

	let values = entry
		.properties
		.iter()
		.enumerate()
		.map(|(i, prop)| {
			let value = use_signal(|| (prop.default_value)());

			use_effect(move || {
				set_child_value(i, value());
			});

			value
		})
		.collect::<Vec<_>>();

	let src = use_hook({
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
		div { class: "dxp-component-view",

			iframe {
				id: CHILD_ID,
				src: "{src}",
				"data-state": if iframe_ready() { "ready" } else { "loading" },
			}

			Panel { entry, iframe_ready, values }
		}
	}
}
