pub mod adapter;
mod bool;
mod r#enum;
mod int;
mod string;

pub use adapter::{use_bool_signal, use_enum_signal, use_int_signal, use_string_signal};
pub use bool::BoolInput;
pub use r#enum::EnumInput;
pub use int::IntInput;
pub use string::StringInput;
