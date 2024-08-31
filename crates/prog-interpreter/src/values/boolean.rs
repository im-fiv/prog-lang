use std::fmt::{self, Debug, Display};
use std::ops::{BitAnd, BitOr};

use super::RPrimitive;

//* Note: `Debug` is implemented manually below
#[derive(Clone, PartialEq)]
pub struct RBoolean(bool);

impl RPrimitive for RBoolean {
	type Inner = bool;

	fn get(&self) -> &Self::Inner { &self.0 }

	fn get_mut(&mut self) -> &mut Self::Inner { &mut self.0 }
}

impl BitAnd for RBoolean {
	type Output = Self;

	fn bitand(self, rhs: Self) -> Self::Output { Self(self.0 & rhs.0) }
}

impl BitOr for RBoolean {
	type Output = Self;

	fn bitor(self, rhs: Self) -> Self::Output { Self(self.0 | rhs.0) }
}

impl From<bool> for RBoolean {
	fn from(value: bool) -> Self { Self(value) }
}

impl Debug for RBoolean {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { Display::fmt(self, f) }
}

impl Display for RBoolean {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", if self.0 { "true" } else { "false" })
	}
}
