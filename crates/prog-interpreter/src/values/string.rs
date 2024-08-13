use std::collections::HashMap;
use std::fmt::{self, Debug, Display};

use anyhow::Result;
use prog_macros::{get_argument, get_this};

use super::{
	CallSite, IntrinsicFunction, RuntimeNumber, RuntimePrimitive, RuntimeValue, RuntimeValueKind
};
use crate::arg_parser::{Arg, ArgList, ParsedArg};
use crate::RuntimeContext;

//* Note: `Debug` is implemented manually below
#[derive(Clone, PartialEq)]
pub struct RuntimeString(pub String);

impl RuntimeString {
	fn sub(
		this: Option<RuntimeValue>,
		_context: &mut RuntimeContext,
		args: HashMap<String, ParsedArg>,
		_call_site: CallSite
	) -> Result<RuntimeValue> {
		let this = get_this!(this => String);
		let this = this.value();

		let this_len = this.len();

		let start_index = get_argument!(args => start: RuntimeNumber).owned() as usize;

		let end_index = get_argument!(args => end: RuntimeNumber?)
			.and_then(|value| Some(value.owned() as usize))
			.unwrap_or(this_len);

		if end_index <= start_index {
			return Ok(RuntimeValue::String("".into()));
		}

		let mut indices = this.char_indices();
		let unwrap_index = |(index, _)| index;

		let start = indices.nth(start_index).map_or(this_len, unwrap_index);

		let end = indices
			.nth(end_index - start_index - 1)
			.map_or(this_len, unwrap_index);

		let substring = unsafe { this.get_unchecked(start..end) };

		Ok(Self::from(substring).into())
	}

	fn len(
		this: Option<RuntimeValue>,
		_context: &mut RuntimeContext,
		_args: HashMap<String, ParsedArg>,
		_call_site: CallSite
	) -> Result<RuntimeValue> {
		let this = get_this!(this => String);
		let len = this.value().len();

		Ok(RuntimeNumber::from(len).into())
	}
}

impl RuntimePrimitive for RuntimeString {
	type Inner = String;

	fn value(&self) -> &Self::Inner { &self.0 }

	fn dispatch_map(&self) -> HashMap<String, IntrinsicFunction> {
		let mut map = HashMap::new();

		map.insert(
			String::from("sub"),
			IntrinsicFunction::new(
				Self::sub,
				ArgList::new(vec![
					Arg::Required("start", RuntimeValueKind::Number),
					Arg::Optional("end", RuntimeValueKind::Number),
				])
			)
		);

		map.insert(
			String::from("len"),
			IntrinsicFunction::new(Self::len, ArgList::new_empty())
		);

		map
	}
}

impl From<String> for RuntimeString {
	fn from(value: String) -> Self { Self(value) }
}

impl From<&str> for RuntimeString {
	fn from(value: &str) -> Self { Self(value.to_owned()) }
}

impl Debug for RuntimeString {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "\"{}\"", self.0) }
}

impl Display for RuntimeString {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}
