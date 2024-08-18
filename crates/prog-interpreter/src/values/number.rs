use std::collections::HashMap;
use std::fmt::{self, Debug, Display};

use super::{RIntrinsicFunction, RPrimitive};

//* Note: `Debug` is implemented manually below
#[derive(Clone, PartialEq)]
pub struct RNumber(f64);

impl RPrimitive for RNumber {
	type Inner = f64;

	fn get(&self) -> &Self::Inner { &self.0 }

	fn get_mut(&mut self) -> &mut Self::Inner { &mut self.0 }

	fn dispatch_map(&self) -> HashMap<String, RIntrinsicFunction> { HashMap::new() }
}

impl From<f64> for RNumber {
	fn from(value: f64) -> Self { Self(value) }
}

impl From<usize> for RNumber {
	fn from(value: usize) -> Self { Self(value as f64) }
}

impl Debug for RNumber {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { Display::fmt(self, f) }
}

impl Display for RNumber {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}
