use std::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq, prog_macros::EnumKind)]
pub enum Value {
	Empty
}

impl Display for Value {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Empty => write!(f, "")
		}
	}
}
