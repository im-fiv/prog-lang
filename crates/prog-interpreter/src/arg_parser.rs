use std::collections::HashMap;
use std::fmt::{self, Display};
use std::ops::Range;

use crate::{Value, ValueKind};

pub type ParsedArgList<'i> = HashMap<String, ParsedArg<'i>>;

#[derive(Debug, Clone, PartialEq)]
pub struct ArgList {
	args: Option<Vec<Arg>>
}

impl<'i> ArgList {
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
			args: Some(arguments)
		}
	}

	/// Creates an argument list that accepts 0 arguments
	pub fn new_empty() -> Self { Self { args: None } }

	/// Verifies the provided arguments according to the inner argument types list
	pub fn verify(&self, arguments: &[Value<'i>]) -> Result<ParsedArgList<'i>, ArgumentParseError> {
		use core::iter::zip;

		if let Some(result) = self.check_args_length(arguments)? {
			return Ok(result);
		}

		let expected_args = self.args.as_ref().unwrap();
		let mut found_args = arguments.to_owned();

		// It is crucial to balance both of the vectors such that the `for` loop actually runs
		if found_args.len() < expected_args.len() {
			found_args.resize(expected_args.len(), Value::None)
		}

		let mut result = HashMap::new();
		let zipped_args = zip(expected_args, found_args.clone());

		for (index, (expected_arg, found_arg)) in zipped_args.enumerate() {
			let mut check_args =
				|expected: ValueKind, found: ValueKind, name: &str, optional: bool| {
					if !optional && (expected != found) {
						return Err(ArgumentParseError::IncorrectType {
							index,
							expected,
							found
						});
					}

					if optional && (found == ValueKind::None) && (expected != ValueKind::None) {
						return Ok(());
					}

					result.insert(String::from(name), ParsedArg::Regular(found_arg.clone()));

					Ok(())
				};

			match expected_arg {
				Arg::Required(name, kind) => check_args(*kind, found_arg.kind(), name, false)?,
				Arg::RequiredUntyped(name) => {
					check_args(found_arg.kind(), found_arg.kind(), name, false)?
				}
				Arg::Optional(name, kind) => check_args(*kind, found_arg.kind(), name, true)?,
				Arg::Variadic(name) => {
					result.insert(
						String::from(name.to_owned()),
						ParsedArg::Variadic(found_args[index..].to_vec())
					);
				}
			};
		}

		Ok(result)
	}

	fn check_args_length(
		&self,
		found: &[Value<'i>]
	) -> Result<Option<ParsedArgList<'i>>, ArgumentParseError> {
		if self.args.is_none() {
			if !found.is_empty() {
				return Err(ArgumentParseError::CountMismatch {
					expected: 0..0,
					end_boundary: true,
					found: found.len()
				});
			}

			return Ok(Some(HashMap::new()));
		}

		let own_args = self.args.as_ref().unwrap();

		let mut num_optional = 0;
		let mut has_variadic = false;

		for arg in own_args {
			match arg {
				Arg::Required(..) | Arg::RequiredUntyped(..) => continue,
				Arg::Optional(..) => num_optional += 1,
				Arg::Variadic(_) => {
					has_variadic = true;
					break;
				}
			};
		}

		let found_len = found.len();
		let expected_len = own_args.len();

		if (found_len != expected_len) && (num_optional == 0) && !has_variadic {
			return Err(ArgumentParseError::CountMismatch {
				expected: expected_len..expected_len,
				end_boundary: true,
				found: found_len
			});
		}

		if !has_variadic
			&& (num_optional > 0)
			&& ((found_len < expected_len - num_optional) || (found_len > expected_len))
		{
			return Err(ArgumentParseError::CountMismatch {
				expected: (expected_len - num_optional)..expected_len,
				end_boundary: true,
				found: found_len
			});
		}

		// Argument list may contain only 1 variadic argument, hence the -1
		if has_variadic && (num_optional == 0) && (found_len < expected_len - 1) {
			let expected_len = expected_len - 1;
			return Err(ArgumentParseError::CountMismatch {
				expected: expected_len..expected_len,
				end_boundary: false,
				found: found_len
			});
		}

		if has_variadic && (num_optional > 0) && (found_len < expected_len - 1 - num_optional) {
			let expected_len = expected_len - 1 - num_optional;
			return Err(ArgumentParseError::CountMismatch {
				expected: expected_len..expected_len,
				end_boundary: false,
				found: found_len
			});
		}

		Ok(None)
	}
}

impl Display for ArgList {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.args.is_none() {
			return write!(f, "");
		}

		let mut arg_strings = vec![];

		for arg in self.args.as_ref().unwrap() {
			let formatted_arg = match arg {
				Arg::Required(name, kind) => format!("{name}: {kind}"),
				Arg::RequiredUntyped(name) => format!("{name}"),
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
	Required(Box<str>, ValueKind),
	RequiredUntyped(Box<str>),
	Optional(Box<str>, ValueKind),
	Variadic(Box<str>)
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedArg<'i> {
	Regular(Value<'i>),
	Variadic(Vec<Value<'i>>)
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArgumentParseError {
	CountMismatch {
		expected: Range<usize>,
		end_boundary: bool,
		found: usize
	},

	IncorrectType {
		index: usize,
		expected: ValueKind,
		found: ValueKind
	}
}
