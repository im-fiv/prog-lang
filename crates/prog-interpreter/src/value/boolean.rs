use std::fmt::{self, Display};
use std::ops::Not;

use crate::Primitive;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Bool(bool);

impl Bool {
	pub fn new(value: bool) -> Self { Self(value) }
}

impl Primitive for Bool {
	fn is_truthy(&self) -> bool { self.0 }
}

impl Not for Bool {
	type Output = Self;

	fn not(self) -> Self::Output { Self(!self.0) }
}

impl Display for Bool {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}
