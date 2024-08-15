use std::fmt::{self, Debug, Display};

use super::RuntimePrimitive;

//* Note: `Debug` is implemented manually below
#[derive(Clone, PartialEq)]
pub struct RuntimeBoolean(pub bool);

impl RuntimePrimitive for RuntimeBoolean {
	type Inner = bool;

	fn get(&self) -> &Self::Inner { &self.0 }

	fn get_mut(&mut self) -> &mut Self::Inner { &mut self.0 }
}

impl From<bool> for RuntimeBoolean {
	fn from(value: bool) -> Self { Self(value) }
}

impl Debug for RuntimeBoolean {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { Display::fmt(self, f) }
}

impl Display for RuntimeBoolean {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", if self.0 { "true" } else { "false" })
	}
}
