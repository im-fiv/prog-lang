use std::collections::HashMap;
use std::fmt::{self, Debug, Display};

use super::{IntrinsicFunction, RuntimePrimitive};

//* Note: `Debug` is implemented manually below
#[derive(Clone, PartialEq)]
pub struct RuntimeNumber(f64);

impl RuntimePrimitive for RuntimeNumber {
	type Inner = f64;

	fn get(&self) -> &Self::Inner { &self.0 }

	fn get_mut(&mut self) -> &mut Self::Inner { &mut self.0 }

	fn dispatch_map(&self) -> HashMap<String, IntrinsicFunction> { HashMap::new() }
}

impl From<f64> for RuntimeNumber {
	fn from(value: f64) -> Self { Self(value) }
}

impl From<usize> for RuntimeNumber {
	fn from(value: usize) -> Self { Self(value as f64) }
}

impl Debug for RuntimeNumber {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { Display::fmt(self, f) }
}

impl Display for RuntimeNumber {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}
