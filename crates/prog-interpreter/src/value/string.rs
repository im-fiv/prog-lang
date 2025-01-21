use std::fmt::{self, Display};

use crate::Primitive;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Str(String);

impl Primitive for Str {
	fn is_truthy(&self) -> bool { !self.0.is_empty() }
}

impl From<String> for Str {
	fn from(value: String) -> Self { Self(value) }
}

impl From<&'_ str> for Str {
	fn from(value: &'_ str) -> Self { Self::from(value.to_string()) }
}

impl From<Str> for String {
	fn from(value: Str) -> Self { value.0 }
}

impl Display for Str {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if f.alternate() {
			write!(f, "\"{}\"", self.0)
		} else {
			write!(f, "{}", self.0)
		}
	}
}
