use std::collections::HashMap;
use std::fmt::Display;

use super::{RuntimePrimitive, IntrinsicFunction};

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeBoolean(pub bool);

impl RuntimePrimitive for RuntimeBoolean {
	type Inner = bool;

	fn uv(self) -> Self::Inner {
		self.0
	}

	fn cv(&self) -> Self::Inner {
		self.0
	}

	fn dispatch_map(&self) -> HashMap<String, IntrinsicFunction> {
		HashMap::new()
	}
}

impl From<bool> for RuntimeBoolean {
	fn from(value: bool) -> Self {
		Self(value)
	}
}

impl Display for RuntimeBoolean {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", if self.0 { "true" } else { "false" })
	}
}