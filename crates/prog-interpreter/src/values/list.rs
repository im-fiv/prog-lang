use std::collections::HashMap;
use std::fmt::{self, Debug, Display};

use anyhow::Result;
use prog_macros::get_this;

use super::{RIntrinsicFunction, RIntrinsicFunctionData, RNumber, RPrimitive, Value};
use crate::arg_parser::ArgList;

//* Note: `Debug` is implemented manually below
#[derive(Clone, PartialEq)]
pub struct RList(Vec<Value>);

impl RList {
	fn len(RIntrinsicFunctionData { this, .. }: RIntrinsicFunctionData) -> Result<Value> {
		let this = get_this!(this => List);
		let len = this.get().len();

		Ok(RNumber::from(len).into())
	}
}

impl RPrimitive for RList {
	type Inner = Vec<Value>;

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

impl From<Vec<Value>> for RList {
	fn from(value: Vec<Value>) -> Self { Self(value) }
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
