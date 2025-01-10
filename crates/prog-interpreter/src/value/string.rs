use std::fmt::{self, Display};

use crate::Primitive;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Str(String);

impl Str {
	pub fn new(value: String) -> Self { Self(value) }
}

impl Primitive for Str {
	fn is_truthy(&self) -> bool { !self.0.is_empty() }
}

impl Display for Str {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}
