use dioxus::prelude::*;

use crate::model::{IntType, IntValue, Value};

/// Creates a [`Signal<bool>`] from a [`Signal<Value>`] holding a [`Value::Bool`].
///
/// Reads the initial boolean from `value` and keeps the two signals in sync:
/// any write to the returned signal is immediately reflected in `value`.
pub fn use_bool_signal<V>(mut value: V) -> Signal<bool>
where
	V: 'static + Copy + PartialEq + Writable<Target = Value>,
{
	let signal = use_signal(|| match *value.peek() {
		Value::Bool(b) => b,
		_ => false,
	});
	use_effect(move || value.set(Value::Bool(signal())));
	signal
}

/// Creates a [`Signal<String>`] from a [`Signal<Value>`] holding a
/// [`Value::String`].
///
/// Reads the initial string from `value` and keeps the two signals in sync:
/// any write to the returned signal is immediately reflected in `value`.
pub fn use_string_signal<V>(mut value: V) -> Signal<String>
where
	V: 'static + Copy + PartialEq + Writable<Target = Value>,
{
	let signal = use_signal(|| match value.peek().clone() {
		Value::String(s) => s,
		_ => String::new(),
	});
	use_effect(move || value.set(Value::String(signal())));
	signal
}

/// Creates a [`Signal<i64>`] from a [`Signal<Value>`] holding a [`Value::Int`].
///
/// `int_type` determines which [`IntValue`] variant is produced when writing
/// back. Reads the initial integer (widened to `i64`) from `value` and keeps
/// the two signals in sync: any write to the returned signal is immediately
/// reflected in `value`.
pub fn use_int_signal<V>(int_type: IntType, mut value: V) -> Signal<i64>
where
	V: 'static + Copy + PartialEq + Writable<Target = Value>,
{
	let signal = use_signal(|| match *value.peek() {
		Value::Int(ref iv) => iv.as_i128() as i64,
		_ => 0,
	});
	use_effect(move || {
		value.set(Value::Int(IntValue::from_i128(signal() as i128, int_type)));
	});
	signal
}

/// Creates a [`Signal<u8>`] from a [`Signal<Value>`] holding a [`Value::Enum`].
///
/// Reads the initial variant index from `value` and keeps the two signals in
/// sync: any write to the returned signal is immediately reflected in `value`.
pub fn use_enum_signal<V>(mut value: V) -> Signal<u8>
where
	V: 'static + Copy + PartialEq + Writable<Target = Value>,
{
	let signal = use_signal(|| match *value.peek() {
		Value::Enum(i) => i,
		_ => 0,
	});
	use_effect(move || value.set(Value::Enum(signal())));
	signal
}
