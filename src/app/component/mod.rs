use std::hash::{DefaultHasher, Hash, Hasher};

use dioxus::prelude::*;

use crate::{
	app::protocol::{notify_child_ready, use_child},
	model::{ComponentEntry, Value},
};

use super::config;

const STYLE: Asset = asset!("src/app/component/style.css");

/// Rendered inside the iframe — just the component plus any configured CSS,
/// no storybook chrome.
#[component]
pub fn ComponentPage(name: String, props: String) -> Element {
	let css = config::css_assets();

	rsx! {
		document::Link { rel: "stylesheet", href: STYLE }
		for asset in css {
			document::Link { rel: "stylesheet", href: asset }
		}
		ComponentBody { name, props }
	}
}

#[component]
fn ComponentBody(name: String, props: String) -> Element {
	let values = use_child(|| {
		serde_json::from_str(&props).unwrap_or_else(|_| {
			inventory::iter::<ComponentEntry>
				.into_iter()
				.find(|e| e.name == name.as_str())
				.map(|e| e.properties.iter().map(|p| (p.default_value)()).collect())
				.unwrap_or_default()
		})
	});

	// Notify the parent that WASM has rendered and the iframe is ready to be
	// shown. Runs once after the first render (use_hook is not reactive).
	use_hook(notify_child_ready);

	let values = values();
	let key = {
		let mut h = DefaultHasher::new();
		values.hash(&mut h);
		h.finish()
	};

	for entry in inventory::iter::<ComponentEntry> {
		if entry.name == name {
			return rsx! {
				Remount {
					key_hash: key,
					{(entry.render)(values)}
				}
			};
		}
	}

	rsx! {
		p {
			style: "color:#9ca3af;padding:40px;font-family:sans-serif;",
			"Component \"{name}\" not found."
		}
	}
}

/// Forces a full remount of its contents whenever `key_hash` changes.
#[component]
fn Remount(key_hash: u64, children: Element) -> Element {
	rsx! {
		for children in [children] {
			Remounted {
				key: "{key_hash}",
				{children}
			}
		}
	}
}

#[component]
fn Remounted(children: Element) -> Element {
	children
}
