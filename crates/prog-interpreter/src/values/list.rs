use std::collections::HashMap;
use std::fmt::{self, Debug, Display};
use std::ops::Index;

use anyhow::Result;
use halloc::HeapMutator;
use prog_macros::get_this;

use super::{RIntrinsicFunction, RIntrinsicFunctionData, RNumber, RPrimitive, Value};
use crate::arg_parser::ArgList;

//* Note: `Debug` and `PartialEq` are implemented manually below
#[derive(Clone)]
pub struct RList(HeapMutator<'static, Vec<Value>>);

impl RList {
	fn len(RIntrinsicFunctionData { this, .. }: RIntrinsicFunctionData) -> Result<Value> {
		let this = get_this!(this => List);
		let len = this.get().len();

		Ok(RNumber::from(len).into())
	}

	// TODO: `insert` and `push`
}

impl RPrimitive for RList {
	type Inner = HeapMutator<'static, Vec<Value>>;

	fn get(&self) -> &Self::Inner { &self.0 }

	fn get_mut(&mut self) -> &mut Self::Inner { &mut self.0 }

	fn dispatch_map(&self) -> HashMap<String, RIntrinsicFunction> {
		let mut map = HashMap::new();

		map.insert(
			String::from("len"),
			RIntrinsicFunction::new(Self::len, ArgList::new_empty())
		);

		map
	}
}

impl PartialEq for RList {
	fn eq(&self, other: &Self) -> bool { self.0.get() == other.0.get() }
}

impl Index<usize> for RList {
	type Output = Value;

	fn index(&self, index: usize) -> &Self::Output { (*self.0).get(index).unwrap_or(&Value::Empty) }
}

impl Index<RNumber> for RList {
	type Output = Value;

	fn index(&self, index: RNumber) -> &Self::Output {
		let index = index.get_owned() as usize;
		&self[index]
	}
}

impl From<HeapMutator<'static, Vec<Value>>> for RList {
	fn from(value: HeapMutator<'static, Vec<Value>>) -> Self { Self(value) }
}

// Same as with Object
impl Drop for RList {
	fn drop(&mut self) {
		if !self.0.can_dealloc() {
			return;
		}

		for value in self.0.drain(..) {
			drop(value);
		}
	}
}

impl Debug for RList {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { Display::fmt(self, f) }
}

impl Display for RList {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let formatted = self
			.0
			.iter()
			.map(|entry| entry.to_string())
			.collect::<Vec<String>>()
			.join(", ");

		write!(f, "[{formatted}]")
	}
}
