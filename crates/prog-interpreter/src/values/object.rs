use std::collections::HashMap;
use std::fmt::{self, Debug, Display};

use halloc::HeapMutator;

use super::{RuntimePrimitive, RuntimeValue};

//* Note: `Debug` and `PartialEq` are implemented manually below
#[derive(Clone)]
pub struct RuntimeObject(pub HeapMutator<'static, HashMap<String, RuntimeValue>>);

impl RuntimePrimitive for RuntimeObject {
	type Inner = HashMap<String, RuntimeValue>;

	fn value(&self) -> &Self::Inner { self.0.get() }
}

impl From<HeapMutator<'static, HashMap<String, RuntimeValue>>> for RuntimeObject {
	fn from(value: HeapMutator<'static, HashMap<String, RuntimeValue>>) -> Self { Self(value) }
}

impl PartialEq for RuntimeObject {
	fn eq(&self, other: &Self) -> bool { *self.0 == *other.0 }
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
