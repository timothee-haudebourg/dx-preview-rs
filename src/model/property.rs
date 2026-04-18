use super::{Reflect, Type, TypeError, Value};

#[derive(Copy, Clone, PartialEq)]
/// Describes a single visible, editable property of a component.
pub struct Property {
	pub name: &'static str,
	pub r#type: Type,
	pub required: bool,
}

pub trait PropertyType: Sized {
	/// Property value type.
	type Type;

	/// Is the property required?
	const REQUIRED: bool;

	fn to_opt_value(&self) -> Option<Value>;

	fn try_from_opt_value(value: Option<Value>) -> Result<Self, TypeError>;
}

impl<T: Reflect> PropertyType for T {
	type Type = Self;

	const REQUIRED: bool = true;

	fn to_opt_value(&self) -> Option<Value> {
		Some(Reflect::to_value(self))
	}

	fn try_from_opt_value(value: Option<Value>) -> Result<Self, TypeError> {
		match value {
			Some(value) => Reflect::try_from_value(value),
			None => Err(TypeError::Required),
		}
	}
}

impl<T: Reflect> PropertyType for Option<T> {
	type Type = T;

	const REQUIRED: bool = false;

	fn to_opt_value(&self) -> Option<Value> {
		self.as_ref().map(|t| t.to_value())
	}

	fn try_from_opt_value(value: Option<Value>) -> Result<Self, TypeError> {
		match value {
			Some(t) => T::try_from_value(t).map(Some),
			None => Ok(None),
		}
	}
}
