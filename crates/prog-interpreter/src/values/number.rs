use std::collections::HashMap;
use std::fmt::{self, Debug, Display};
use std::ops::{Add, Div, Mul, Rem, Sub};

use super::{RIntrinsicFunction, RPrimitive};

#[derive(Clone, PartialEq, PartialOrd)]
pub struct RNumber(f64);

impl RPrimitive for RNumber {
	type Inner = f64;

	fn get(&self) -> &Self::Inner { &self.0 }

	fn get_mut(&mut self) -> &mut Self::Inner { &mut self.0 }

	fn dispatch_map(&self) -> HashMap<String, RIntrinsicFunction> { HashMap::new() }
}

impl Add for RNumber {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output { Self(self.0 + rhs.0) }
}

impl Sub for RNumber {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output { Self(self.0 - rhs.0) }
}

impl Div for RNumber {
	type Output = Self;

	fn div(self, rhs: Self) -> Self::Output { Self(self.0 / rhs.0) }
}

impl Mul for RNumber {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output { Self(self.0 * rhs.0) }
}

impl Rem for RNumber {
	type Output = Self;

	fn rem(self, rhs: Self) -> Self::Output { Self(self.0 % rhs.0) }
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
