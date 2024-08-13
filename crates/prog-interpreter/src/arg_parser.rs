use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::ops::Range;

use crate::{RuntimeValue, RuntimeValueKind};

//* Note: `Debug` is implemented manually below
#[derive(Clone, PartialEq)]
pub struct ArgList {
	arguments: Option<Vec<Arg>>
}

impl ArgList {
	/// Creates an argument list from the provided arguments
	pub fn new(arguments: Vec<Arg>) -> Self {
		let mut variadic_count = 0;
		let mut met_optional = false;

		for (index, arg) in arguments.iter().enumerate() {
			match arg {
				Arg::Variadic(_) => variadic_count += 1,
				Arg::Optional(..) => met_optional = true,

				_ => {
					if (variadic_count > 0) && (index != arguments.len() - 1) {
						panic!("Variadic arguments must always be at the end of the argument list");
					}

					if met_optional {
						panic!(
							"Optional arguments must always be positioned after required arguments"
						);
					}
				}
			}
		}

		if variadic_count > 1 {
			panic!("Only 1 variadic argument must be present in an argument list");
		}

		Self {
			arguments: Some(arguments)
		}
	}

	/// Creates an argument list that accepts 0 arguments
	pub fn new_empty() -> Self { Self { arguments: None } }

	/// Verifies the provided arguments according to the inner argument types list
	pub fn verify(
		&self,
		arguments: &[RuntimeValue]
	) -> Result<HashMap<String, ParsedArg>, ArgumentParseError> {
		use core::iter::zip;

		if let Some(result) = self.check_args_length(arguments)? {
			return Ok(result);
		}

		let own_arguments = self.arguments.as_ref().unwrap();
		let mut arguments = arguments.to_owned();

		// It is crucial to balance both of the vectors such that the `for` loop actually runs
		if arguments.len() < own_arguments.len() {
			arguments.resize(own_arguments.len(), RuntimeValue::Empty)
		}

		let mut result = HashMap::new();

		for (index, (own_argument, got_argument)) in
			zip(own_arguments, arguments.clone()).enumerate()
		{
			let mut check_args = |expected: &RuntimeValueKind,
			                      got: &RuntimeValueKind,
			                      name: &str,
			                      optional: bool| {
				if !optional && (expected != got) {
					return Err(ArgumentParseError::incorrect_type(
						index,
						expected.to_string(),
						got.to_string()
					));
				}

				if optional
					&& (got == &RuntimeValueKind::Empty)
					&& (expected != &RuntimeValueKind::Empty)
				{
					return Ok(());
				}

				result.insert(String::from(name), ParsedArg::Regular(got_argument.clone()));

				Ok(())
			};

			match own_argument {
				Arg::Required(name, kind) => check_args(kind, &got_argument.kind(), name, false)?,
				Arg::Optional(name, kind) => check_args(kind, &got_argument.kind(), name, true)?,
				Arg::Variadic(name) => {
					result.insert(
						String::from(name.to_owned()),
						ParsedArg::Variadic(arguments[index..].to_vec())
					);
				}
			};
		}

		Ok(result)
	}

	fn check_args_length(
		&self,
		got: &[RuntimeValue]
	) -> Result<Option<HashMap<String, ParsedArg>>, ArgumentParseError> {
		if self.arguments.is_none() {
			if !got.is_empty() {
				return Err(ArgumentParseError::count_mismatch(0..0, true, got.len()));
			}

			return Ok(Some(HashMap::new()));
		}

		let own_arguments = self.arguments.as_ref().unwrap();

		let mut num_optional = 0;
		let mut has_variadic = false;

		for arg in own_arguments {
			match arg {
				Arg::Required(..) => continue,
				Arg::Optional(..) => num_optional += 1,
				Arg::Variadic(_) => {
					has_variadic = true;
					break;
				}
			};
		}

		let got_len = got.len();
		let expected_len = own_arguments.len();

		if (got_len != expected_len) && (num_optional == 0) && !has_variadic {
			return Err(ArgumentParseError::count_mismatch(
				expected_len..expected_len,
				true,
				got_len
			));
		}

		if !has_variadic
			&& (num_optional > 0)
			&& ((got_len < expected_len - num_optional) || (got_len > expected_len))
		{
			return Err(ArgumentParseError::count_mismatch(
				(expected_len - num_optional)..expected_len,
				true,
				got_len
			));
		}

		// Argument list may contain only 1 variadic argument, hence the -1
		if has_variadic && (num_optional == 0) && (got_len < expected_len - 1) {
			let expected_len = expected_len - 1;
			return Err(ArgumentParseError::count_mismatch(
				expected_len..expected_len,
				false,
				got_len
			));
		}

		if has_variadic && (num_optional > 0) && (got_len < expected_len - 1 - num_optional) {
			let expected_len = expected_len - 1 - num_optional;
			return Err(ArgumentParseError::count_mismatch(
				expected_len..expected_len,
				false,
				got_len
			));
		}

		Ok(None)
	}
}

impl Debug for ArgList {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.arguments.is_none() {
			return write!(f, "");
		}

		let mut arg_strings = vec![];

		for arg in self.arguments.as_ref().unwrap() {
			let formatted_arg = match arg {
				Arg::Required(name, kind) => format!("{name}: {kind}"),
				Arg::Optional(name, kind) => format!("{name}: {kind}?"),
				Arg::Variadic(name) => format!("{name}...")
			};

			arg_strings.push(formatted_arg);
		}

		write!(f, "{}", arg_strings.join(", "))
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum Arg {
	Required(&'static str, RuntimeValueKind),
	Optional(&'static str, RuntimeValueKind),
	Variadic(&'static str)
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedArg {
	Regular(RuntimeValue),
	Variadic(Vec<RuntimeValue>)
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArgumentParseError {
	CountMismatch {
		expected: Range<usize>,
		end_boundary: bool,
		got: usize
	},

	IncorrectType {
		index: usize,
		expected: String,
		got: String
	}
}

impl ArgumentParseError {
	pub fn count_mismatch(expected: Range<usize>, end_boundary: bool, got: usize) -> Self {
		Self::CountMismatch {
			expected,
			end_boundary,
			got
		}
	}

	pub fn incorrect_type(index: usize, expected: String, got: String) -> Self {
		Self::IncorrectType {
			index,
			expected,
			got
		}
	}
}
