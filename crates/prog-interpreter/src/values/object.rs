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

// This is necessary since it is important to deallocate the inner elements
// **before** the main `HeapMutator`. Otherwise, the result would look something like this:
//
// 1. `drop()` called on `RuntimeObject`
// 2. `drop()` called on `HeapMutator<HashMap<_, _>>`
// 3. `HeapMutator` tries to drop each element, while having the heap lock
// 4. Each element that is an object also tries to acquire a heap lock, which is already acquired by the parent
// 5. The element's `HeapMutator` keeps waiting for the heap lock indefinitely
//
// * Additional note: while the interpreter handles references as owned values,
// * the cloned objects will constantly try to deallocate themselves when going out of scope,
// * despite them only being a clone
impl Drop for RuntimeObject {
	fn drop(&mut self) {
		if !self.0.can_dealloc() {
			return;
		}

		for (_, value) in self.0.drain() {
			// `name` will be dropped automatically
			drop(value);
		}
	}
}

impl Debug for RuntimeObject {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut debug_struct = f.debug_struct(&format!("Object ({} refs)", self.0.ref_count()));

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
