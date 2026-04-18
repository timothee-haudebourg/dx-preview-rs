//! Data model shared between the shell and the isolation page.
//!
//! The central type is [`ComponentEntry`], which is registered at link time by
//! the [`preview`](crate::preview) attribute macro and collected via
//! [`inventory`]. The shell iterates over all collected entries to populate the
//! sidebar and property editor.

pub use inventory;

/// A component registered at compile time by the [`preview`](crate::preview) attribute.
///
/// Each annotated component contributes one `ComponentEntry` to the
/// [`inventory`] registry. The shell reads this registry on startup to
/// populate the sidebar.
pub struct ComponentEntry {
	/// Name of the component.
	pub name: &'static str,

	/// Component properties.
	pub properties: &'static [Property],

	/// Default value for each property.
	pub default_values: fn() -> Vec<Value>,

	/// Render the component with the given properties.
	pub render: fn(Vec<Value>) -> dioxus::prelude::Element,
}

impl PartialEq for ComponentEntry {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name
	}
}

inventory::collect!(ComponentEntry);

// ── ShowcaseType implementations ─────────────────────────────────────────────

impl ShowcaseType for bool {
	const TYPE: Type = Type::Bool;

	fn to_value(&self) -> Value {
		Value::Bool(*self)
	}

	fn try_from_value(value: Value) -> Result<Self, TypeError> {
		match value {
			Value::Bool(b) => Ok(b),
			_ => Err(TypeError),
		}
	}
}

impl ShowcaseType for String {
	const TYPE: Type = Type::String;

	fn to_value(&self) -> Value {
		Value::String(self.clone())
	}

	fn try_from_value(value: Value) -> Result<Self, TypeError> {
		match value {
			Value::String(s) => Ok(s),
			_ => Err(TypeError),
		}
	}
}

macro_rules! impl_showcase_int {
	($prim:ty, $int_type:ident, $int_value:ident) => {
		impl ShowcaseType for $prim {
			const TYPE: Type = Type::Int(IntType::$int_type);

			fn to_value(&self) -> Value {
				Value::Int(IntValue::$int_value(*self))
			}

			fn try_from_value(value: Value) -> Result<Self, TypeError> {
				match value {
					Value::Int(IntValue::$int_value(n)) => Ok(n),
					_ => Err(TypeError),
				}
			}
		}
	};
}

impl_showcase_int!(u8, U8, U8);
impl_showcase_int!(u16, U16, U16);
impl_showcase_int!(u32, U32, U32);
impl_showcase_int!(u64, U64, U64);
impl_showcase_int!(i8, I8, I8);
impl_showcase_int!(i16, I16, I16);
impl_showcase_int!(i32, I32, I32);
impl_showcase_int!(i64, I64, I64);

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Copy, Clone, PartialEq)]
/// Describes a single visible, editable property of a component.
pub struct Property {
	pub name: &'static str,
	pub r#type: Type,
	pub required: bool,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Type {
	Bool,
	Int(IntType),
	String,
}

#[derive(Copy, Clone, PartialEq)]
pub enum IntType {
	U8,
	U16,
	U32,
	U64,
	I8,
	I16,
	I32,
	I64,
}

// ── Values ────────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Value {
	Bool(bool),
	Int(IntValue),
	String(String),
}

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum IntValue {
	U8(u8),
	U16(u16),
	U32(u32),
	U64(u64),
	I8(i8),
	I16(i16),
	I32(i32),
	I64(i64),
}

impl IntValue {
	/// Returns the inner value widened to `i128` for display/editing.
	pub fn as_i128(&self) -> i128 {
		match self {
			IntValue::U8(n) => *n as i128,
			IntValue::U16(n) => *n as i128,
			IntValue::U32(n) => *n as i128,
			IntValue::U64(n) => *n as i128,
			IntValue::I8(n) => *n as i128,
			IntValue::I16(n) => *n as i128,
			IntValue::I32(n) => *n as i128,
			IntValue::I64(n) => *n as i128,
		}
	}

	/// Converts an `i128` back to the variant matching `int_type`.
	pub fn from_i128(n: i128, int_type: IntType) -> Self {
		match int_type {
			IntType::U8 => IntValue::U8(n as u8),
			IntType::U16 => IntValue::U16(n as u16),
			IntType::U32 => IntValue::U32(n as u32),
			IntType::U64 => IntValue::U64(n as u64),
			IntType::I8 => IntValue::I8(n as i8),
			IntType::I16 => IntValue::I16(n as i16),
			IntType::I32 => IntValue::I32(n as i32),
			IntType::I64 => IntValue::I64(n as i64),
		}
	}
}

// ── Trait ─────────────────────────────────────────────────────────────────────

#[derive(Copy, Clone)]
pub struct TypeError;

pub trait ShowcaseType: Sized {
	const TYPE: Type;

	fn to_value(&self) -> Value;

	fn try_from_value(value: Value) -> Result<Self, TypeError>;
}
