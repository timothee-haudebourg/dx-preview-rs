use serde::{Deserialize, Serialize};
use std::fmt;

use crate::model::{EnumType, Reflect, Type, TypeError};

use super::IntType;

pub mod signal;
pub mod source;

pub use signal::*;
pub use source::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
	Bool(bool),
	Int(IntValue),
	String(String),
	Enum(u8),
	/// An optional value; `None` means the property is absent.
	Option(Option<Box<Value>>),
	/// A reactive value shared between the child and parent windows.
	Signal(ReactiveValue),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EnumValue(pub u8);

impl Reflect for EnumValue {
	const TYPE: Type = Type::Enum(EnumType {
		name: "EnumValue",
		variants: &[],
	});

	fn to_value(self) -> Value {
		Value::Enum(self.0)
	}

	fn try_from_value(value: Value) -> Result<Self, TypeError> {
		match value {
			Value::Enum(n) => Ok(Self(n)),
			_ => Err(TypeError::InvalidType),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

impl fmt::Display for IntValue {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			IntValue::U8(n) => n.fmt(f),
			IntValue::U16(n) => n.fmt(f),
			IntValue::U32(n) => n.fmt(f),
			IntValue::U64(n) => n.fmt(f),
			IntValue::I8(n) => n.fmt(f),
			IntValue::I16(n) => n.fmt(f),
			IntValue::I32(n) => n.fmt(f),
			IntValue::I64(n) => n.fmt(f),
		}
	}
}

impl fmt::Display for Value {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use dioxus::signals::ReadableExt;
		match self {
			Value::Bool(b) => b.fmt(f),
			Value::Int(i) => i.fmt(f),
			Value::String(s) => s.fmt(f),
			Value::Enum(n) => n.fmt(f),
			Value::Option(None) => write!(f, "None"),
			Value::Option(Some(v)) => write!(f, "Some({v})"),
			Value::Signal(rv) => write!(f, "Signal({})", &*rv.peek()),
		}
	}
}
