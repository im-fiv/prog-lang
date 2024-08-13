use std::collections::HashMap;
use std::fmt::{self, Debug, Display};

use super::{IntrinsicFunction, RuntimePrimitive, RuntimeValue};

//* Note: `Debug` is implemented manually below
#[derive(Clone, PartialEq)]
pub struct RuntimeObject(pub HashMap<String, RuntimeValue>);

impl RuntimePrimitive for RuntimeObject {
	type Inner = HashMap<String, RuntimeValue>;

	fn value(&self) -> &Self::Inner { &self.0 }

	fn dispatch_map(&self) -> HashMap<String, IntrinsicFunction> {
		// TODO
		todo!(
			"implement behavior for when an object has a user-defined function with the same name"
		)
	}
}

impl From<HashMap<String, RuntimeValue>> for RuntimeObject {
	fn from(value: HashMap<String, RuntimeValue>) -> Self { Self(value) }
}

impl Debug for RuntimeObject {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut debug_struct = f.debug_struct("Object");

		for (name, value) in self.0.iter() {
			debug_struct.field(name, value);
		}

		debug_struct.finish()
	}
}

impl Display for RuntimeObject {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let formatted = self
			.0
			.iter()
			.map(|(name, value)| format!("{name} = {value}"))
			.collect::<Vec<String>>()
			.join(", ");

		write!(f, "{{ {formatted} }}")
	}
}
