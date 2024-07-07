use std::fmt::Display;

use super::RuntimeValue;

#[derive(Debug, Clone, PartialEq)]
pub enum MarkerKind {
	Return(Box<RuntimeValue>),
	Break,
	Continue
}

impl Display for MarkerKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Return(value) => write!(f, "return {value}"),
			Self::Break => write!(f, "break"),
			Self::Continue => write!(f, "continue")
		}
	}
}
