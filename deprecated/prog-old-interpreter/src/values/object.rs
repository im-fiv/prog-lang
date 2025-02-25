use std::collections::HashMap;
use std::fmt::{self, Debug, Display};

use halloc::HeapMutator;

use super::{RPrimitive, Value};

#[derive(Clone)]
pub struct RObject(HeapMutator<'static, HashMap<String, Value>>);

impl RPrimitive for RObject {
	type Inner = HeapMutator<'static, HashMap<String, Value>>;

	fn get(&self) -> &Self::Inner { &self.0 }

	fn get_mut(&mut self) -> &mut Self::Inner { &mut self.0 }
}

impl From<HeapMutator<'static, HashMap<String, Value>>> for RObject {
	fn from(value: HeapMutator<'static, HashMap<String, Value>>) -> Self { Self(value) }
}

impl PartialEq for RObject {
	fn eq(&self, other: &Self) -> bool { *self.0 == *other.0 }
}

// This is necessary since it is important to deallocate the inner elements
// **before** the main `HeapMutator`. Otherwise, the result would look something like this:
//
// 1. `drop()` called on `RObject`
// 2. `drop()` called on `HeapMutator<HashMap<_, _>>`
// 3. `HeapMutator` tries to drop each element, while having the heap lock
// 4. Each element that is an object also tries to acquire a heap lock, which is already acquired by the parent
// 5. The element's `HeapMutator` keeps waiting for the heap lock indefinitely
//
// * Additional note: while the interpreter handles references as owned values,
// * the cloned objects will constantly try to deallocate themselves when going out of scope,
// * despite them only being a clone
impl Drop for RObject {
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

impl Debug for RObject {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut debug_struct = f.debug_struct(&format!("Object ({} refs)", self.0.ref_count()));

		for (name, value) in self.0.iter() {
			debug_struct.field(name, value);
		}

		debug_struct.finish()
	}
}

impl Display for RObject {
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
