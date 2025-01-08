use std::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq, prog_macros::EnumKind)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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
