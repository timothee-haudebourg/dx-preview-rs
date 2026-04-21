use dioxus::signals::Signal;

use super::{IntValue, ReactiveValue, Value};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
	Bool,
	Int(IntType),
	String,
	Enum(EnumType),
	Option(&'static Self),
	Signal(&'static Self),
}

impl Type {
	pub fn default_value(&self) -> Value {
		match self {
			Self::Bool => Value::Bool(false),
			Self::Int(t) => Value::Int(t.default_value()),
			Self::String => Value::String(String::new()),
			Self::Enum(t) => Value::Enum(t.default_value()),
			Self::Option(_) => Value::Option(None),
			Self::Signal(t) => Value::Signal(ReactiveValue::new(t.default_value())),
		}
	}
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

impl IntType {
	pub fn default_value(&self) -> IntValue {
		match self {
			Self::U8 => IntValue::U8(0),
			Self::U16 => IntValue::U16(0),
			Self::U32 => IntValue::U32(0),
			Self::U64 => IntValue::U64(0),
			Self::I8 => IntValue::I8(0),
			Self::I16 => IntValue::I16(0),
			Self::I32 => IntValue::I32(0),
			Self::I64 => IntValue::I64(0),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EnumType {
	pub name: &'static str,
	pub variants: &'static [EnumVariant],
}

impl EnumType {
	pub fn default_value(&self) -> u8 {
		0
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EnumVariant {
	pub name: &'static str,
}

#[derive(Copy, Clone)]
pub enum TypeError {
	InvalidType,
	InvalidValue,
	Required,
}

pub trait Reflect: Sized {
	const TYPE: Type;

	fn to_value(self) -> Value;

	fn try_from_value(value: Value) -> Result<Self, TypeError>;
}

impl Reflect for bool {
	const TYPE: Type = Type::Bool;

	fn to_value(self) -> Value {
		Value::Bool(self)
	}

	fn try_from_value(value: Value) -> Result<Self, TypeError> {
		match value {
			Value::Bool(b) => Ok(b),
			_ => Err(TypeError::InvalidType),
		}
	}
}

impl Reflect for String {
	const TYPE: Type = Type::String;

	fn to_value(self) -> Value {
		Value::String(self)
	}

	fn try_from_value(value: Value) -> Result<Self, TypeError> {
		match value {
			Value::String(s) => Ok(s),
			_ => Err(TypeError::InvalidType),
		}
	}
}

impl<T> Reflect for Option<T>
where
	T: Reflect,
{
	const TYPE: Type = Type::Option(&T::TYPE);

	fn to_value(self) -> Value {
		Value::Option(self.map(|t| Box::new(t.to_value())))
	}

	fn try_from_value(value: Value) -> Result<Self, TypeError> {
		match value {
			Value::Option(Some(t)) => Ok(Some(T::try_from_value(*t)?)),
			Value::Option(None) => Ok(None),
			_ => Err(TypeError::InvalidType),
		}
	}
}

impl<T> Reflect for Signal<T>
where
	T: 'static + PartialEq + Reflect + Clone,
{
	const TYPE: Type = Type::Signal(&T::TYPE);

	fn to_value(self) -> Value {
		Value::Signal(ReactiveValue::use_from_signal::<T>(self))
	}

	fn try_from_value(value: Value) -> Result<Self, TypeError> {
		match value {
			Value::Signal(rv) => Ok(rv.use_adapter::<T>()),
			_ => Err(TypeError::InvalidType),
		}
	}
}

macro_rules! impl_showcase_int {
	($prim:ty, $int_type:ident, $int_value:ident) => {
		impl Reflect for $prim {
			const TYPE: Type = Type::Int(IntType::$int_type);

			fn to_value(self) -> Value {
				Value::Int(IntValue::$int_value(self))
			}

			fn try_from_value(value: Value) -> Result<Self, TypeError> {
				match value {
					Value::Int(IntValue::$int_value(n)) => Ok(n),
					_ => Err(TypeError::InvalidType),
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
