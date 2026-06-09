use dioxus::prelude::*;

use crate::{
	app::ui::{BoolInput, EnumInput, IntInput, StringInput},
	model::{AnyValueSignal, EnumType, IntType, Property, ReactiveValue, Type, Value},
};

const STYLE: Asset = asset!("./style.css");

#[component]
pub fn PropertyEditor(prop: &'static Property, value: Signal<Value>) -> Element {
	let type_expr = prop.type_expr.to_string();

	rsx! {
		div { class: "dxp-property-editor",

			document::Link { rel: "stylesheet", href: STYLE }

			div { class: "dxp-header",
				label { "{prop.name}" }
				div { "{type_expr}" }
			}

			ValueEditor { ty: &prop.r#type, value }
		}
	}
}

// ── ValueEditor: dispatch to the right per-type editor ────────────────────────

/// Dispatch to the right typed editor based on `ty`.
#[component]
fn ValueEditor<V>(ty: &'static Type, value: V) -> Element
where
	V: PartialEq + AnyValueSignal,
{
	match ty {
		Type::Bool => rsx! {
			BoolPropertyEditor { value }
		},
		Type::String => rsx! {
			StringPropertyEditor { value }
		},
		Type::Int(int_type) => rsx! {
			IntPropertyEditor { int_type: *int_type, value }
		},
		Type::Enum(enum_type) => rsx! {
			EnumPropertyEditor { enum_type: *enum_type, value }
		},
		Type::Option(inner) => rsx! {
			OptionPropertyEditor { inner, value }
		},
		Type::Signal(inner_ty) => rsx! {
			SignalPropertyEditor { inner_ty, value }
		},
	}
}

#[component]
fn BoolPropertyEditor<V>(value: V) -> Element
where
	V: PartialEq + AnyValueSignal,
{
	let value = use_hook(|| value.couple_into_signal().unwrap());
	rsx! {
		BoolInput { value }
	}
}

#[component]
fn StringPropertyEditor<V>(value: V) -> Element
where
	V: PartialEq + AnyValueSignal,
{
	let value = use_hook(|| value.couple_into_signal().unwrap());
	rsx! {
		StringInput { value }
	}
}

#[component]
fn IntPropertyEditor<V>(int_type: IntType, value: V) -> Element
where
	V: PartialEq + AnyValueSignal,
{
	match int_type {
		IntType::U8 => {
			let value: Signal<u8> = use_hook(|| value.couple_into_signal().unwrap());
			rsx! { IntInput { value } }
		}
		IntType::U16 => {
			let value: Signal<u16> = use_hook(|| value.couple_into_signal().unwrap());
			rsx! { IntInput { value } }
		}
		IntType::U32 => {
			let value: Signal<u32> = use_hook(|| value.couple_into_signal().unwrap());
			rsx! { IntInput { value } }
		}
		IntType::U64 => {
			let value: Signal<u64> = use_hook(|| value.couple_into_signal().unwrap());
			rsx! { IntInput { value } }
		}
		IntType::I8 => {
			let value: Signal<i8> = use_hook(|| value.couple_into_signal().unwrap());
			rsx! { IntInput { value } }
		}
		IntType::I16 => {
			let value: Signal<i16> = use_hook(|| value.couple_into_signal().unwrap());
			rsx! { IntInput { value } }
		}
		IntType::I32 => {
			let value: Signal<i32> = use_hook(|| value.couple_into_signal().unwrap());
			rsx! { IntInput { value } }
		}
		IntType::I64 => {
			let value: Signal<i64> = use_hook(|| value.couple_into_signal().unwrap());
			rsx! { IntInput { value } }
		}
	}
}

#[component]
fn EnumPropertyEditor<V>(enum_type: EnumType, value: V) -> Element
where
	V: PartialEq + AnyValueSignal,
{
	let value = use_hook(|| value.couple_into_signal().unwrap());
	rsx! {
		EnumInput { variants: enum_type.variants, value }
	}
}

/// Editor for `Option<T>`: a checkbox to toggle Some/None, plus the inner
/// editor when the value is Some.
#[component]
fn OptionPropertyEditor<V>(inner: &'static Type, mut value: V) -> Element
where
	V: 'static + Clone + PartialEq + Writable<Target = Value>,
{
	let mut is_some = use_signal(|| matches!(*value.peek(), Value::Option(Some(_))));

	let inner_value = use_signal(|| match value.peek().clone() {
		Value::Option(Some(v)) => *v,
		_ => inner.default_value(),
	});

	// (is_some, inner_value) → value
	use_effect(move || {
		value.set(if is_some() {
			Value::Option(Some(Box::new(inner_value())))
		} else {
			Value::Option(None)
		});
	});

	rsx! {
		div { class: "dxp-option-editor",
			input {
				r#type: "checkbox",
				checked: is_some(),
				onchange: move |e| is_some.set(e.checked()),
			}
			if is_some() {
				ValueEditor { ty: inner, value: inner_value }
			}
		}
	}
}

/// Editor for `Signal<T>`: shows the inner editor with a ⚡ icon, bridging
/// the property's [`Value::Signal`] to a plain [`Signal<Value>`] that the
/// inner editor can read and write.
#[component]
fn SignalPropertyEditor<V>(inner_ty: &'static Type, value: V) -> Element
where
	V: 'static + Clone + PartialEq + Writable<Target = Value>,
{
	let inner_value: ReactiveValue = match value.peek().clone() {
		Value::Signal(rv) => rv,
		_ => panic!("SignalPropertyEditor: expected Value::Signal"),
	};

	rsx! {
		div { class: "dxp-signal-editor",
			span { title: "Reactive signal — changes are shared with the preview",
				"⚡"
			}
			ValueEditor { ty: inner_ty, value: inner_value }
		}
	}
}
