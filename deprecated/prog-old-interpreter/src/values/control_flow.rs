use std::fmt::Display;

use super::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum ControlFlow {
	Return(Box<Value>),
	Break,
	Continue
}

impl Display for ControlFlow {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Return(value) => write!(f, "return {value}"),
			Self::Break => write!(f, "break"),
			Self::Continue => write!(f, "continue")
		}
	}
}
