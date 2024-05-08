use std::collections::HashMap;
use std::fmt::Display;

use super::{RuntimePrimitive, IntrinsicFunction};

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeNumber(pub f64);

impl RuntimePrimitive for RuntimeNumber {
	type Inner = f64;

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

impl From<f64> for RuntimeNumber {
	fn from(value: f64) -> Self {
		Self(value)
	}
}

impl From<usize> for RuntimeNumber {
	fn from(value: usize) -> Self {
		Self(value as f64)
	}
}

impl Display for RuntimeNumber {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}