use std::fmt::{self, Display};
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

use crate::{AsRaw, Primitive};

// TODO: change to an enum with `Integer` and `Float` as variants
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Num(f64);

impl Primitive for Num {
	fn is_truthy(&self) -> bool { self.0 != 0.0 }
}

impl AsRaw for Num {
	type Inner = f64;

	fn as_raw(&self) -> &Self::Inner { &self.0 }
}

impl PartialEq<f64> for Num {
	fn eq(&self, other: &f64) -> bool { self.0 == *other }
}

impl Add for Num {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output { Self(self.0 + rhs.0) }
}

impl Sub for Num {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output { Self(self.0 - rhs.0) }
}

impl Mul for Num {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output { Self(self.0 * rhs.0) }
}

impl Div for Num {
	type Output = Self;

	fn div(self, rhs: Self) -> Self::Output { Self(self.0 / rhs.0) }
}

impl Rem for Num {
	type Output = Self;

	fn rem(self, rhs: Self) -> Self::Output { Self(self.0 % rhs.0) }
}

impl Neg for Num {
	type Output = Self;

	fn neg(self) -> Self::Output { Self(-self.0) }
}

impl From<f64> for Num {
	fn from(value: f64) -> Self { Self(value) }
}

impl From<Num> for f64 {
	fn from(value: Num) -> Self { value.0 }
}

impl Display for Num {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}
