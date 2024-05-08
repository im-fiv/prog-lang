use std::collections::HashMap;
use std::fmt::Display;
use anyhow::Result;
use prog_macros::get_this;

use crate::arg_parser::{ArgList, ParsedArg};
use crate::RuntimeContext;
use super::{RuntimePrimitive, RuntimeValue, IntrinsicFunction, CallSite};

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeList(pub Vec<RuntimeValue>);

impl RuntimeList {
	pub fn len(
		this: Option<Box<RuntimeValue>>,
		_context: &mut RuntimeContext,
		_args: HashMap<String, ParsedArg>,
		_call_site: CallSite
	) -> Result<RuntimeValue> {
		let this = get_this!(this => List).uv();
		Ok(RuntimeValue::Number(this.len().into()))
	}
}

impl RuntimePrimitive for RuntimeList {
	type Inner = Vec<RuntimeValue>;

	fn uv(self) -> Self::Inner {
		self.0
	}

	fn cv(&self) -> Self::Inner {
		self.0.to_owned()
	}

	fn dispatch_map(&self) -> HashMap<String, IntrinsicFunction> {
		let mut map = HashMap::new();

		map.insert(String::from("len"), IntrinsicFunction::new(
			Self::len,
			ArgList::new_empty()
		));

		map
	}
}

impl From<Vec<RuntimeValue>> for RuntimeList {
	fn from(value: Vec<RuntimeValue>) -> Self {
		Self(value)
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