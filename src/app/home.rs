use std::rc::Rc;

use dioxus::prelude::*;

use crate::book::{ComponentEntry, IntType, IntValue, Property, Type, Value};

use super::{IFRAME_ID, IframeSender, Route, use_iframe_ready};

// ── Context ───────────────────────────────────────────────────────────────────

struct Context {
	current_index: Signal<Option<usize>>,
	entries: Vec<&'static ComponentEntry>,
}

impl Context {
	fn read_current_entry(&self) -> Option<&'static ComponentEntry> {
		self.current_index
			.read()
			.and_then(|i| self.entries.get(i).copied())
	}
}

// ── Home ──────────────────────────────────────────────────────────────────────

#[component]
pub fn Home() -> Element {
	let context = provide_context(Rc::new(Context {
		current_index: use_signal(|| None),
		entries: inventory::iter::<ComponentEntry>.into_iter().collect(),
	}));

	rsx! {
		div {
			style: "display:flex;height:100vh;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;font-size:14px;color:#111827;",

			Menu {}

			main {
				style: "flex:1;overflow:hidden;background:#fff;display:flex;flex-direction:column;",
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
			style: "display:flex;flex-direction:column;align-items:center;justify-content:center;height:100%;gap:8px;color:#9ca3af;",
			span { style: "font-size:36px;", "📖" }
			span { "Select a component from the sidebar" }
		}
	}
}

// ── Menu ──────────────────────────────────────────────────────────────────────

#[component]
fn Menu() -> Element {
	let context: Rc<Context> = use_context();

	rsx! {
		nav {
			style: "width:240px;min-width:240px;border-right:1px solid #e5e7eb;overflow-y:auto;background:#f9fafb;display:flex;flex-direction:column;",

			header {
				style: "padding:16px;font-weight:700;font-size:15px;border-bottom:1px solid #e5e7eb;letter-spacing:-0.3px;flex-shrink:0;",
				"dx·preview"
			}

			div {
				style: "padding:8px 0;",
				for (index, entry) in context.entries.iter().enumerate() {
					MenuItem { index, entry }
				}
			}
		}
	}
}

#[component]
fn MenuItem(index: usize, entry: &'static ComponentEntry) -> Element {
	let context: Rc<Context> = use_context();
	let mut selected = context.current_index;
	let is_selected = use_memo(move || selected() == Some(index));

	rsx! {
		div {
			style: if is_selected() {
				"padding:8px 16px;cursor:pointer;border-radius:6px;margin:1px 6px;user-select:none;background:#2563eb;color:#fff;"
			} else {
				"padding:8px 16px;cursor:pointer;border-radius:6px;margin:1px 6px;user-select:none;background:none;color:#374151;"
			},
			onclick: move |_| selected.set(Some(index)),
			"{entry.name}"
		}
	}
}

// ── ComponentView ─────────────────────────────────────────────────────────────

#[component]
fn ComponentView(entry: &'static ComponentEntry) -> Element {
	// Initialise values from the component's own defaults.
	let values: Signal<Vec<Value>> = use_signal(entry.default_values);

	// src is fixed for the lifetime of this ComponentView instance (the `key`
	// on the call site ensures a fresh mount whenever `entry` changes).
	// We include the initial default values so the iframe can render correctly
	// even before the first postMessage arrives.
	let src = {
		let initial_json = serde_json::to_string(&(entry.default_values)()).unwrap_or_default();
		Route::ComponentPage {
			name: entry.name.to_string(),
			props: initial_json,
		}
		.to_string()
	};

	// Becomes true when the iframe sends its Ready message (WASM rendered,
	// CSS applied). Controls visibility to eliminate the FOUC.
	let iframe_ready = use_iframe_ready();
	let sender = IframeSender;

	// Send current values whenever the iframe becomes ready or values change.
	use_effect(move || {
		let is_ready = iframe_ready();
		let vals = values.read().clone();
		if is_ready {
			sender.send_values(&vals);
		}
	});

	let iframe_style = if iframe_ready() {
		"flex:1;width:100%;height:100%;border:none;transition:opacity 150ms ease;opacity:1;"
	} else {
		"flex:1;width:100%;height:100%;border:none;transition:opacity 150ms ease;opacity:0;"
	};

	rsx! {
		div {
			style: "height:100%;display:flex;",

			iframe {
				id: IFRAME_ID,
				src: "{src}",
				style: iframe_style,
			}

			ComponentProperties { entry, values }
		}
	}
}

// ── ComponentProperties ───────────────────────────────────────────────────────

#[component]
fn ComponentProperties(entry: &'static ComponentEntry, values: Signal<Vec<Value>>) -> Element {
	if entry.properties.is_empty() {
		return rsx! {
			div {
				style: "width:260px;min-width:260px;border-left:1px solid #e5e7eb;display:flex;align-items:center;justify-content:center;color:#9ca3af;font-size:13px;padding:16px;text-align:center;",
				"No editable properties"
			}
		};
	}

	rsx! {
		div {
			style: "width:260px;min-width:260px;border-left:1px solid #e5e7eb;overflow-y:auto;background:#fafafa;display:flex;flex-direction:column;",

			header {
				style: "padding:12px 16px;font-weight:600;font-size:13px;border-bottom:1px solid #e5e7eb;color:#374151;letter-spacing:0.01em;",
				"Properties"
			}

			div {
				style: "padding:8px 0;",
				for (index, prop) in entry.properties.iter().enumerate() {
					PropertyEditor { prop, index, values }
				}
			}
		}
	}
}

// ── PropertyEditor ────────────────────────────────────────────────────────────

#[component]
fn PropertyEditor(prop: &'static Property, index: usize, values: Signal<Vec<Value>>) -> Element {
	rsx! {
		div {
			style: "padding:8px 16px;",

			label {
				style: "display:block;font-size:12px;font-weight:500;color:#6b7280;margin-bottom:4px;",
				"{prop.name}"
			}

			match prop.r#type {
				Type::Bool => rsx! {
					BoolEditor { index, values }
				},
				Type::String => rsx! {
					StringEditor { index, values }
				},
				Type::Int(int_type) => rsx! {
					IntEditor { index, int_type, values }
				},
			}
		}
	}
}

// ── Field editors ─────────────────────────────────────────────────────────────

#[component]
fn BoolEditor(index: usize, mut values: Signal<Vec<Value>>) -> Element {
	let checked = use_memo(move || matches!(values.read().get(index), Some(Value::Bool(true))));

	rsx! {
		label {
			style: "display:flex;align-items:center;gap:8px;cursor:pointer;",
			input {
				r#type: "checkbox",
				checked: checked(),
				style: "width:16px;height:16px;cursor:pointer;accent-color:#2563eb;",
				onchange: move |e| {
					if let Some(slot) = values.write().get_mut(index) {
						*slot = Value::Bool(e.checked());
					}
				},
			}
			span {
				style: "font-size:13px;color:#111827;",
				if checked() { "true" } else { "false" }
			}
		}
	}
}

#[component]
fn StringEditor(index: usize, mut values: Signal<Vec<Value>>) -> Element {
	let current = use_memo(move || match values.read().get(index) {
		Some(Value::String(s)) => s.clone(),
		_ => String::new(),
	});

	rsx! {
		input {
			r#type: "text",
			value: "{current}",
			style: "width:100%;box-sizing:border-box;border:1px solid #d1d5db;border-radius:4px;padding:5px 8px;font-size:13px;outline:none;background:#fff;",
			oninput: move |e| {
				if let Some(slot) = values.write().get_mut(index) {
					*slot = Value::String(e.value());
				}
			},
		}
	}
}

#[component]
fn IntEditor(index: usize, int_type: IntType, mut values: Signal<Vec<Value>>) -> Element {
	let current = use_memo(move || match values.read().get(index) {
		Some(Value::Int(iv)) => iv.as_i128().to_string(),
		_ => "0".to_string(),
	});

	rsx! {
		input {
			r#type: "number",
			value: "{current}",
			style: "width:100%;box-sizing:border-box;border:1px solid #d1d5db;border-radius:4px;padding:5px 8px;font-size:13px;outline:none;background:#fff;",
			oninput: move |e| {
				if let Ok(n) = e.value().parse::<i128>()
					&& let Some(slot) = values.write().get_mut(index) {
						*slot = Value::Int(IntValue::from_i128(n, int_type));
					}
			},
		}
	}
}
