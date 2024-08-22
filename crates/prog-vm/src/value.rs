use std::cmp::Ordering;
use std::ops::*;

use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};

use crate::{Instruction, VM};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
	Boolean(bool),
	String(String),
	Number(f64),
	Function {
		arity: usize,
		instructions: Vec<Instruction>
	},
	#[serde(skip)]
	IntrinsicFunction {
		arity: Option<usize>,
		pointer: fn(&mut VM) -> Result<()>
	},
	Empty
}

impl Value {
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

			v => todo!("{v}")
		}
	}
}

impl Add for Value {
	type Output = Result<Value>;

	fn add(self, rhs: Self) -> Self::Output {
		match (self, rhs) {
			(Self::Number(lhs), Self::Number(rhs)) => Ok(Value::Number(lhs + rhs)),

			(Self::String(lhs), Self::Boolean(rhs)) => Ok(Value::String(format!("{lhs}{rhs}"))),
			(Self::String(lhs), Self::String(rhs)) => Ok(Value::String(format!("{lhs}{rhs}"))),
			(Self::String(lhs), Self::Number(rhs)) => Ok(Value::String(format!("{lhs}{rhs}"))),

			(lhs, rhs) => bail!("Cannot perform binary add on `{lhs:?}` and `{rhs:?}`")
		}
	}
}

impl Sub for Value {
	type Output = Result<Value>;

	fn sub(self, rhs: Self) -> Self::Output {
		match (self, rhs) {
			(Self::Number(lhs), Self::Number(rhs)) => Ok(Value::Number(lhs - rhs)),

			(lhs, rhs) => bail!("Cannot perform binary sub on `{lhs:?}` and `{rhs:?}`")
		}
	}
}

impl Mul for Value {
	type Output = Result<Value>;

	fn mul(self, rhs: Self) -> Self::Output {
		match (self, rhs) {
			(Self::Number(lhs), Self::Number(rhs)) => Ok(Value::Number(lhs * rhs)),

			(lhs, rhs) => bail!("Cannot perform binary mul on `{lhs:?}` and `{rhs:?}`")
		}
	}
}

impl Div for Value {
	type Output = Result<Value>;

	fn div(self, rhs: Self) -> Self::Output {
		match (self, rhs) {
			(Self::Number(lhs), Self::Number(rhs)) => Ok(Value::Number(lhs / rhs)),

			(lhs, rhs) => bail!("Cannot perform binary div on `{lhs:?}` and `{rhs:?}`")
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

impl Neg for Value {
	type Output = Result<Value>;

	fn neg(self) -> Self::Output {
		match self {
			Self::Number(v) => Ok(Self::Number(-v)),
			v => bail!("Cannot perform unary neg on `{v:?}`")
		}
	}
}

impl Not for Value {
	type Output = Value;

	fn not(self) -> Self::Output {
		Value::Boolean(match self {
			Self::Boolean(v) => !v,
			Self::Number(v) => v == 0.0,
			Self::String(v) => v.is_empty(),
			Self::Function { .. } => false,
			Self::IntrinsicFunction { .. } => false,
			Self::Empty => true
		})
	}
}
