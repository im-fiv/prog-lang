use std::fmt::Display;
use super::{HashMap, RuntimeValue};

pub trait RuntimePrimitive {
	type Inner;

	/// Unwraps inner value of the primitive
	fn uv(self) -> Self::Inner;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeBoolean(pub bool);

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeString(pub String);

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeNumber(pub f64);

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeList(pub Vec<RuntimeValue>);

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeObject(pub HashMap<String, RuntimeValue>);

//* From<T> *//

impl From<bool> for RuntimeBoolean {
	fn from(value: bool) -> Self {
		Self(value)
	}
}

impl From<String> for RuntimeString {
	fn from(value: String) -> Self {
		Self(value)
	}
}

impl From<f64> for RuntimeNumber {
	fn from(value: f64) -> Self {
		Self(value)
	}
}

impl From<Vec<RuntimeValue>> for RuntimeList {
	fn from(value: Vec<RuntimeValue>) -> Self {
		Self(value)
	}
}

impl From<HashMap<String, RuntimeValue>> for RuntimeObject {
	fn from(value: HashMap<String, RuntimeValue>) -> Self {
		Self(value)
	}
}

//* Display *//

impl Display for RuntimeBoolean {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", if self.0 { "true" } else { "false" })
	}
}

impl Display for RuntimeString {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl Display for RuntimeNumber {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl Display for RuntimeList {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let formatted = self.0
			.iter()
			.map(|entry| entry.to_string())
			.collect::<Vec<String>>()
			.join(", ");
		
		write!(f, "[{formatted}]")
	}
}

impl Display for RuntimeObject {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let formatted = self.0
			.iter()
			.map(|(name, value)| format!("{name} = {value}"))
			.collect::<Vec<String>>()
			.join(", ");
		
		write!(f, "{{ {formatted} }}")
	}
}

//* RuntimePrimitive *//

impl RuntimePrimitive for RuntimeBoolean {
	type Inner = bool;

	fn uv(self) -> Self::Inner {
		self.0
	}
}

impl RuntimePrimitive for RuntimeString {
	type Inner = String;

	fn uv(self) -> Self::Inner {
		self.0
	}
}

impl RuntimePrimitive for RuntimeNumber {
	type Inner = f64;

	fn uv(self) -> Self::Inner {
		self.0
	}
}

impl RuntimePrimitive for RuntimeList {
	type Inner = Vec<RuntimeValue>;

	fn uv(self) -> Self::Inner {
		self.0
	}
}

impl RuntimePrimitive for RuntimeObject {
	type Inner = HashMap<String, RuntimeValue>;

	fn uv(self) -> Self::Inner {
		self.0
	}
}