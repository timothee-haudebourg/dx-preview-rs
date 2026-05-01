use std::rc::Rc;

use dioxus::prelude::*;

use super::ui::{Menu, THEME};
use crate::model::ComponentEntry;

mod component;

use component::*;

const STYLE: Asset = asset!("src/app/home/style.css");

// ── Context ───────────────────────────────────────────────────────────────────

struct Context {
	current_index: Signal<usize>,
	entries: Vec<&'static ComponentEntry>,
}

impl Context {
	fn read_current_entry(&self) -> Option<&'static ComponentEntry> {
		self.entries.get(*self.current_index.read()).copied()
	}
}

// ── Home ──────────────────────────────────────────────────────────────────────

#[component]
pub fn Home() -> Element {
	let mut entries: Vec<_> = inventory::iter::<ComponentEntry>.into_iter().collect();
	entries.sort_unstable_by_key(|entry| entry.name);

	let context = provide_context(Rc::new(Context {
		current_index: use_signal(|| 0),
		entries,
	}));

	let current_index = context.current_index;
	let items = context
		.entries
		.iter()
		.map(|e| e.name.to_string())
		.collect::<Vec<_>>();

	rsx! {
		document::Link { rel: "stylesheet", href: THEME }
		document::Link { rel: "stylesheet", href: STYLE }

		div {
			class: "dxp-home",

			Menu {
				title: "dx·preview",
				items,
				selected: current_index,
			}

			main {
				if let Some(entry) = context.read_current_entry() {
					ComponentView { key: "{entry.name}", entry }
				} else {
					EmptyView {}
				}
			}
		}
	}
}

#[component]
fn EmptyView() -> Element {
	rsx! {
		div {
			class: "dxp-empty-view",
			span { "📖" }
			span { "Select a component from the sidebar" }
		}
	}
}
