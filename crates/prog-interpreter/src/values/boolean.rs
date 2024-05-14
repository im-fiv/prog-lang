use std::collections::HashMap;
use std::fmt::Display;
use anyhow::Result;
use prog_macros::get_this;

use crate::arg_parser::{ArgList, ParsedArg};
use crate::RuntimeContext;
use super::{RuntimePrimitive, RuntimeValue, IntrinsicFunction, CallSite};

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeBoolean(pub bool);

impl RuntimeBoolean {
	fn invert(
		this: Option<Box<RuntimeValue>>,
		_context: &mut RuntimeContext,
		_args: HashMap<String, ParsedArg>,
		_call_site: CallSite
	) -> Result<RuntimeValue> {
		let this = get_this!(this => Boolean);
		let mut this_borrowed = this.borrow_mut();

		this_borrowed.0 = !this_borrowed.0;

		Ok(RuntimeValue::Empty)
	}
}

impl RuntimePrimitive for RuntimeBoolean {
	type Inner = bool;

	fn value(&self) -> &Self::Inner {
		&self.0
	}

	fn dispatch_map(&self) -> HashMap<String, IntrinsicFunction> {
		let mut map = HashMap::new();

		map.insert(String::from("invert"), IntrinsicFunction::new(
			Self::invert,
			ArgList::new_empty()
		));

		map
	}
}

impl From<bool> for RuntimeBoolean {
	fn from(value: bool) -> Self {
		Self(value)
	}
}

impl Display for RuntimeBoolean {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", if self.0 { "true" } else { "false" })
	}
}