use std::cmp::Ordering;

use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};

use crate::VM;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
	Boolean(bool),
	String(String),
	Number(f64),
	Function {
		arity: usize,
		start: usize,
		length: usize
	},
	Empty,

	// * Important note: any variants that serde should skip must be below all other variants
	#[serde(skip)]
	IntrinsicFunction {
		arity: Option<usize>,
		pointer: fn(&mut VM) -> Result<()>
	}
}

impl Value {
	pub fn add(self, rhs: Self) -> Result<Self> {
		match (self, rhs) {
			(Self::Number(lhs), Self::Number(rhs)) => Ok(Value::Number(lhs + rhs)),

			(Self::String(lhs), Self::Boolean(rhs)) => Ok(Value::String(format!("{lhs}{rhs}"))),
			(Self::String(lhs), Self::String(rhs)) => Ok(Value::String(format!("{lhs}{rhs}"))),
			(Self::String(lhs), Self::Number(rhs)) => Ok(Value::String(format!("{lhs}{rhs}"))),

			(lhs, rhs) => bail!("Cannot perform binary add on `{lhs:?}` and `{rhs:?}`")
		}
	}

	pub fn sub(self, rhs: Self) -> Result<Self> {
		match (self, rhs) {
			(Self::Number(lhs), Self::Number(rhs)) => Ok(Value::Number(lhs - rhs)),

			(lhs, rhs) => bail!("Cannot perform binary sub on `{lhs:?}` and `{rhs:?}`")
		}
	}

	pub fn mul(self, rhs: Self) -> Result<Self> {
		match (self, rhs) {
			(Self::Number(lhs), Self::Number(rhs)) => Ok(Value::Number(lhs * rhs)),

			(lhs, rhs) => bail!("Cannot perform binary mul on `{lhs:?}` and `{rhs:?}`")
		}
	}

	pub fn div(self, rhs: Self) -> Result<Self> {
		match (self, rhs) {
			(Self::Number(lhs), Self::Number(rhs)) => Ok(Value::Number(lhs / rhs)),

			(lhs, rhs) => bail!("Cannot perform binary div on `{lhs:?}` and `{rhs:?}`")
		}
	}

	pub fn neg(self) -> Result<Self> {
		match self {
			Self::Number(v) => Ok(Self::Number(-v)),
			v => bail!("Cannot perform unary neg on `{v:?}`")
		}
	}

	pub fn truthy(&self) -> bool {
		match self {
			Self::Boolean(v) => *v,
			Self::Number(v) => *v != 0.0,
			Self::String(v) => !v.is_empty(),
			Self::Function { .. } => true,
			Self::IntrinsicFunction { .. } => true,
			Self::Empty => false
		}
	}

	pub fn not(self) -> Self { Value::Boolean(!self.truthy()) }

	pub fn custom_partial_cmp(&self, other: &Self) -> Result<Ordering> {
		self.partial_cmp(other)
			.ok_or(anyhow!("Cannot compare `{self:?}` and `{other:?}`"))
	}

	pub fn gt(&self, other: &Self) -> Result<Value> {
		let ordering = self.custom_partial_cmp(other)?;

		Ok(Value::Boolean(matches!(ordering, Ordering::Greater)))
	}

	pub fn lt(&self, other: &Self) -> Result<Value> {
		let ordering = self.custom_partial_cmp(other)?;

		Ok(Value::Boolean(matches!(ordering, Ordering::Less)))
	}

	pub fn gte(&self, other: &Self) -> Result<Value> {
		let ordering = self.custom_partial_cmp(other)?;

		Ok(Value::Boolean(matches!(
			ordering,
			Ordering::Greater | Ordering::Equal
		)))
	}

	pub fn lte(&self, other: &Self) -> Result<Value> {
		let ordering = self.custom_partial_cmp(other)?;

		Ok(Value::Boolean(matches!(
			ordering,
			Ordering::Less | Ordering::Equal
		)))
	}
}

impl std::fmt::Display for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Boolean(v) => write!(f, "{v}"),
			Self::String(v) => write!(f, "{v}"),
			Self::Number(v) => write!(f, "{v}"),
			Self::Empty => write!(f, "none"),

			v => todo!("{v}")
		}
	}
}

impl PartialOrd for Value {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		match (self, other) {
			(Self::Number(lhs), Self::Number(rhs)) => lhs.partial_cmp(rhs),

			_ => None
		}
	}
}
