use std::collections::HashMap;
use std::fmt::{self, Debug, Display};

use anyhow::Result;
use prog_macros::{get_argument, get_this};

use super::{RIntrinsicFunction, RIntrinsicFunctionData, RNumber, RPrimitive, Value, ValueKind};
use crate::arg_parser::{Arg, ArgList, ParsedArg};

//* Note: `Debug` is implemented manually below
#[derive(Clone, PartialEq)]
pub struct RString(String);

impl RString {
	fn sub(
		RIntrinsicFunctionData {
			this, arguments, ..
		}: RIntrinsicFunctionData
	) -> Result<Value> {
		let this = get_this!(this => String);
		let this = this.get();

		let this_len = this.len();

		let start_index = get_argument!(arguments => start: RNumber).get_owned() as usize;

		let end_index = get_argument!(arguments => end: RNumber?)
			.and_then(|value| Some(value.get_owned() as usize))
			.unwrap_or(this_len);

		if end_index <= start_index {
			return Ok(Value::String("".into()));
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

	fn len(RIntrinsicFunctionData { this, .. }: RIntrinsicFunctionData) -> Result<Value> {
		let this = get_this!(this => String);
		let len = this.get().len();

		Ok(RNumber::from(len).into())
	}
}

impl RPrimitive for RString {
	type Inner = String;

	fn get(&self) -> &Self::Inner { &self.0 }

	fn get_mut(&mut self) -> &mut Self::Inner { &mut self.0 }

	fn dispatch_map(&self) -> HashMap<String, RIntrinsicFunction> {
		let mut map = HashMap::new();

		map.insert(
			String::from("sub"),
			RIntrinsicFunction::new(
				Self::sub,
				ArgList::new(vec![
					Arg::Required("start", ValueKind::Number),
					Arg::Optional("end", ValueKind::Number),
				]),
				false
			)
		);

		map.insert(
			String::from("len"),
			RIntrinsicFunction::new(Self::len, ArgList::new_empty(), false)
		);

		map
	}
}

impl From<String> for RString {
	fn from(value: String) -> Self { Self(value) }
}

impl From<&str> for RString {
	fn from(value: &str) -> Self { Self(value.to_owned()) }
}

impl Debug for RString {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "\"{}\"", self.0) }
}

impl Display for RString {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}
