use std::collections::HashMap;
use std::fmt::Display;
use std::cell::RefCell;
use anyhow::Result;
use prog_macros::{get_argument, get_this};

use crate::arg_parser::{ArgList, ParsedArg, Arg};
use crate::RuntimeContext;
use super::{RuntimePrimitive, RuntimeValue, RuntimeValueKind, RuntimeNumber, IntrinsicFunction, CallSite};

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeString(pub String);

impl RuntimeString {
	fn sub(
		this: Option<Box<RuntimeValue>>,
		_context: &mut RuntimeContext,
		args: HashMap<String, ParsedArg>,
		_call_site: CallSite
	) -> Result<RuntimeValue> {
		let this = get_this!(this => String)
			.borrow()
			.owned();

		let this_len = this.len();
		
		let start_index = get_argument!(args => start: RefCell<RuntimeNumber>)
			.borrow()
			.owned()
			as usize;

		let end_index = get_argument!(args => end: RefCell<RuntimeNumber>?)
			.and_then(|value| Some(value
				.borrow()
				.owned()
				as usize
			))
			.unwrap_or(this_len);

		if end_index <= start_index {
			return Ok(RuntimeValue::String(
				Self::from("").into()
			));
		}

		let mut indices = this.char_indices();
		let unwrap_index = |(index, _)| index;

		let start = indices
			.nth(start_index)
			.map_or(this_len, unwrap_index);

		let end = indices
			.nth(end_index - start_index - 1)
			.map_or(this_len, unwrap_index);

		let substring = unsafe {
			this.get_unchecked(start..end)
		};

		Ok(RuntimeValue::String(
			Self::from(substring).into()
		))
	}

	fn len(
		this: Option<Box<RuntimeValue>>,
		_context: &mut RuntimeContext,
		_args: HashMap<String, ParsedArg>,
		_call_site: CallSite
	) -> Result<RuntimeValue> {
		let this = get_this!(this => String);
		let len = this
			.borrow()
			.value()
			.len();

		Ok(RuntimeValue::Number(
			RuntimeNumber::from(len).into()
		))
	}
}

impl RuntimePrimitive for RuntimeString {
	type Inner = String;

	fn value(&self) -> &Self::Inner {
		&self.0
	}

	fn dispatch_map(&self) -> HashMap<String, IntrinsicFunction> {
		let mut map = HashMap::new();

		map.insert(String::from("sub"), IntrinsicFunction::new(
			Self::sub,
			ArgList::new(vec![
				Arg::Required("start", RuntimeValueKind::Number),
				Arg::Optional("end", RuntimeValueKind::Number)
			])
		));

		map.insert(String::from("len"), IntrinsicFunction::new(
			Self::len,
			ArgList::new_empty()
		));

		map
	}
}

impl From<String> for RuntimeString {
	fn from(value: String) -> Self {
		Self(value)
	}
}

impl From<&str> for RuntimeString {
	fn from(value: &str) -> Self {
		Self(value.to_owned())
	}
}

impl Display for RuntimeString {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}