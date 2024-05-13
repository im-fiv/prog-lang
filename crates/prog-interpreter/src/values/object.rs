use std::collections::HashMap;
use std::fmt::Display;

use super::{RuntimePrimitive, RuntimeValue, IntrinsicFunction};

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeObject(pub HashMap<String, RuntimeValue>);

impl RuntimePrimitive for RuntimeObject {
	type Inner = HashMap<String, RuntimeValue>;

	fn value(&self) -> &Self::Inner {
		&self.0
	}

	fn dispatch_map(&self) -> HashMap<String, IntrinsicFunction> {
		// TODO
		todo!("implement behavior for when an object has a user-defined function with the same name")
	}
}

impl From<HashMap<String, RuntimeValue>> for RuntimeObject {
	fn from(value: HashMap<String, RuntimeValue>) -> Self {
		Self(value)
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