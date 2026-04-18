use super::{IntValue, Value};

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

#[derive(Copy, Clone)]
pub enum TypeError {
	InvalidType,
	InvalidValue,
	Required,
}

pub trait Reflect: Sized {
	const TYPE: Type;

	fn to_value(&self) -> Value;

	fn try_from_value(value: Value) -> Result<Self, TypeError>;
}

impl Reflect for bool {
	const TYPE: Type = Type::Bool;

	fn to_value(&self) -> Value {
		Value::Bool(*self)
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

	fn to_value(&self) -> Value {
		Value::String(self.clone())
	}

	fn try_from_value(value: Value) -> Result<Self, TypeError> {
		match value {
			Value::String(s) => Ok(s),
			_ => Err(TypeError::InvalidType),
		}
	}
}

macro_rules! impl_showcase_int {
	($prim:ty, $int_type:ident, $int_value:ident) => {
		impl Reflect for $prim {
			const TYPE: Type = Type::Int(IntType::$int_type);

			fn to_value(&self) -> Value {
				Value::Int(IntValue::$int_value(*self))
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
