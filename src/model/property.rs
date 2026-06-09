use super::{Type, TypeExpr, Value};

#[derive(Copy, Clone)]
/// Describes a single visible, editable property of a component.
pub struct Property {
	pub name: &'static str,
	pub r#type: Type,
	pub type_expr: TypeExpr,
	pub required: bool,
	pub default_value: fn() -> Value,
}

impl PartialEq for Property {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name && self.r#type == other.r#type && self.required == other.required
	}
}
