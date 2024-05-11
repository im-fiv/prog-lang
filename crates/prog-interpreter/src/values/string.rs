use std::collections::HashMap;
use std::fmt::Display;
use anyhow::Result;
use prog_macros::{get_argument, get_this};

use crate::arg_parser::{ArgList, ParsedArg, Arg};
use crate::RuntimeContext;
use super::{RuntimePrimitive, RuntimeValue, RuntimeValueKind, RuntimeNumber, CallSite, IntrinsicFunction};

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeString(pub String);

impl RuntimeString {
	fn sub(
		this: Option<Box<RuntimeValue>>,
		_context: &mut RuntimeContext,
		args: HashMap<String, ParsedArg>,
		_call_site: CallSite
	) -> Result<RuntimeValue> {
		let this = get_this!(this => String).uv();
		
		let start_index = get_argument!(args => start: RuntimeNumber).uv() as usize;
		let end_index = get_argument!(args => end: RuntimeNumber?)
			.and_then(|value| Some(value.uv() as usize))
			.unwrap_or(this.len());

		if end_index <= start_index {
			return Ok(RuntimeValue::String("".into()));
		}

		let mut indices = this.char_indices();
		let unwrap_index = |(index, _)| index;
		let string_len = this.len();

		let start = indices
			.nth(start_index)
			.map_or(string_len, &unwrap_index);

		let end = indices
			.nth(end_index - start_index - 1)
			.map_or(string_len, &unwrap_index);

		let substring = unsafe {
			this.get_unchecked(start..end)
		};

		Ok(RuntimeValue::String(substring.into()))
	}

	fn len(
		this: Option<Box<RuntimeValue>>,
		_context: &mut RuntimeContext,
		_args: HashMap<String, ParsedArg>,
		_call_site: CallSite
	) -> Result<RuntimeValue> {
		let this = get_this!(this => String).uv();
		Ok(RuntimeValue::Number(this.len().into()))
	}
}

impl RuntimePrimitive for RuntimeString {
	type Inner = String;

	fn uv(self) -> Self::Inner {
		self.0
	}

	fn cv(&self) -> Self::Inner {
		self.0.to_owned()
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