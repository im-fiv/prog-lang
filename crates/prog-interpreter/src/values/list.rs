use std::collections::HashMap;
use std::fmt::{self, Debug, Display};

use anyhow::Result;
use prog_macros::get_this;

use super::{CallSite, IntrinsicFunction, RuntimeNumber, RuntimePrimitive, RuntimeValue};
use crate::arg_parser::{ArgList, ParsedArg};
use crate::RuntimeContext;

//* Note: `Debug` is implemented manually below
#[derive(Clone, PartialEq)]
pub struct RuntimeList(pub Vec<RuntimeValue>);

impl RuntimeList {
	fn len(
		this: Option<RuntimeValue>,
		_context: &mut RuntimeContext,
		_args: HashMap<String, ParsedArg>,
		_call_site: CallSite
	) -> Result<RuntimeValue> {
		let this = get_this!(this => List);
		let len = this.value().len();

		Ok(RuntimeNumber::from(len).into())
	}
}

impl RuntimePrimitive for RuntimeList {
	type Inner = Vec<RuntimeValue>;

	fn value(&self) -> &Self::Inner { &self.0 }

	fn dispatch_map(&self) -> HashMap<String, IntrinsicFunction> {
		let mut map = HashMap::new();

		map.insert(
			String::from("len"),
			IntrinsicFunction::new(Self::len, ArgList::new_empty())
		);

		map
	}
}

impl From<Vec<RuntimeValue>> for RuntimeList {
	fn from(value: Vec<RuntimeValue>) -> Self { Self(value) }
}

impl Debug for RuntimeList {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { Display::fmt(self, f) }
}

impl Display for RuntimeList {
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
