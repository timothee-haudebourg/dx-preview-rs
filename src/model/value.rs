use super::IntType;

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Value {
	Bool(bool),
	Int(IntValue),
	String(String),
	Enum(u8),
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
