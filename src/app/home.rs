use std::rc::Rc;

use dioxus::prelude::*;

use super::ui::input::{
	BoolInput, EnumInput, IntInput, StringInput,
	adapter::{use_bool_signal, use_enum_signal, use_int_signal, use_string_signal},
};
use crate::model::{ComponentEntry, EnumType, IntType, Property, Type, Value};

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
	let iframe_ready = use_iframe_ready();
	let sender = IframeSender;

	let iframe_style = if iframe_ready() {
		"flex:1;width:100%;height:100%;border:none;transition:opacity 150ms ease;opacity:1;"
	} else {
		"flex:1;width:100%;height:100%;border:none;transition:opacity 150ms ease;opacity:0;"
	};

	let context = provide_context(Rc::new(PropertiesContext {
		properties: entry
			.properties
			.iter()
			.map(|prop| PropertyState {
				value: use_signal(|| {
					(prop.default_value)().unwrap_or_else(|| prop.r#type.default_value())
				}),
				enabled: use_signal(|| prop.required),
			})
			.collect(),
	}));

	// Memo that combines the stable values with the enabled flags to produce
	// the Vec<Option<Value>> that the iframe expects.
	let combined = use_memo(move || {
		context
			.properties
			.iter()
			.map(|state| {
				if (state.enabled)() {
					Some((state.value)())
				} else {
					None
				}
			})
			.collect::<Vec<_>>()
	});

	// Initial iframe URL — uses the default optional values; subsequent changes
	// are pushed via postMessage without reloading the iframe.
	let src = {
		let json = serde_json::to_string(&*combined.peek()).unwrap_or_default();
		Route::ComponentPage {
			name: entry.name.to_string(),
			props: json,
		}
		.to_string()
	};

	// Push the combined values to the iframe whenever they change and the
	// iframe is ready.
	use_effect(move || {
		if iframe_ready() {
			sender.send_values(&combined.read());
		}
	});

	rsx! {
		div {
			style: "height:100%;display:flex;",

			iframe {
				id: IFRAME_ID,
				src: "{src}",
				style: iframe_style,
			}

			ComponentProperties { entry }
		}
	}
}

// ── ComponentProperties ───────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
struct PropertyState {
	value: Signal<Value>,
	enabled: Signal<bool>,
}

struct PropertiesContext {
	properties: Vec<PropertyState>,
}

#[component]
fn ComponentProperties(entry: &'static ComponentEntry) -> Element {
	let context: Rc<PropertiesContext> = use_context();

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
				for (i, prop) in entry.properties.iter().enumerate() {
					PropertyEditor {
						prop,
						state: context.properties[i]
					}
				}
			}
		}
	}
}

// ── PropertyEditor ────────────────────────────────────────────────────────────

#[component]
fn PropertyEditor(prop: &'static Property, state: PropertyState) -> Element {
	let PropertyState { value, mut enabled } = state;

	rsx! {
		div {
			style: "padding:8px 16px;",

			div {
				style: "display:flex;align-items:center;justify-content:space-between;margin-bottom:4px;",
				label {
					style: if enabled() {
						"font-size:12px;font-weight:500;color:#6b7280;"
					} else {
						"font-size:12px;font-weight:500;color:#d1d5db;"
					},
					"{prop.name}"
				}
				if !prop.required {
					input {
						r#type: "checkbox",
						checked: enabled(),
						style: "width:14px;height:14px;cursor:pointer;accent-color:#2563eb;",
						onchange: move |e| {
							enabled.set(e.checked());
						},
					}
				}
			}

			if enabled() {
				match prop.r#type {
					Type::Bool => rsx! { BoolPropertyEditor { value } },
					Type::String => rsx! { StringPropertyEditor { value } },
					Type::Int(int_type) => rsx! { IntPropertyEditor { int_type, value } },
					Type::Enum(enum_type) => {
						rsx! { EnumPropertyEditor { enum_type, value } }
					}
				}
			}
		}
	}
}

// ── Per-type property editors (adapter → typed input) ─────────────────────────

#[component]
fn BoolPropertyEditor(value: Signal<Value>) -> Element {
	let value = use_bool_signal(value);
	rsx! { BoolInput { value } }
}

#[component]
fn StringPropertyEditor(value: Signal<Value>) -> Element {
	let value = use_string_signal(value);
	rsx! { StringInput { value } }
}

#[component]
fn IntPropertyEditor(int_type: IntType, value: Signal<Value>) -> Element {
	let value = use_int_signal(int_type, value);
	rsx! { IntInput { value } }
}

#[component]
fn EnumPropertyEditor(enum_type: EnumType, value: Signal<Value>) -> Element {
	let value = use_enum_signal(value);
	rsx! { EnumInput { variants: enum_type.variants, value } }
}
