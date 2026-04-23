use core::fmt;

use super::Value;

#[derive(
	Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum ValueSource {
	Child,
	Parent,
}

impl ValueSource {
	pub fn name(&self) -> &'static str {
		match self {
			Self::Child => "child",
			Self::Parent => "parent",
		}
	}

	pub fn is_child(&self) -> bool {
		matches!(self, Self::Child)
	}

	pub fn is_parent(&self) -> bool {
		matches!(self, Self::Parent)
	}
}

impl fmt::Display for ValueSource {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.name().fmt(f)
	}
}

#[derive(Debug, Clone)]
pub struct SourcedValue {
	pub value: Value,
	pub source: ValueSource,
}

impl SourcedValue {
	pub fn new(value: Value, source: ValueSource) -> Self {
		Self { value, source }
	}
}
