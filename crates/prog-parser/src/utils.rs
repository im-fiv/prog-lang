use pest::iterators::Pair;

use super::{expressions, Rule};

macro_rules! assert_rule {
	($var:ident == $rule:ident $(| $rest:ident)* in $main_pair:expr) => {
		if !matches!($var.as_rule(), Rule::$rule $(| Rule::$rest)*) {
			let expected_str = assert_rule!(format_expected $rule $( $rest ),*);

			error!(
				"invalid pair of type `{:?}` in `{:?}` (expected {})", $var.as_span(),
				$var.as_rule(),
				$main_pair.as_rule(),
				expected_str
			);
		}
	};

	(format_expected $rule:ident $($rest:ident),*) => {
		[
			format!("`{:?}`", Rule::$rule),
			$( format!(", `{:?}`", Rule::$rest) ),*
		].concat()
	};
}

macro_rules! get_pair_safe {
	(from $pairs:ident expect $rule:ident $(| $rest:ident)* in $main_pair:expr) => {{
		let expected_str = assert_rule!(format_expected $rule $( $rest ),*);

		let next_pair = $pairs
			.next()
			.unwrap_or_else(|| error!(
				"pair of type {} is missing in `{:?}`",
				$main_pair.as_span(),
				expected_str,
				$main_pair.as_rule()
			));

		assert_rule!(next_pair == $rule $(| $rest)* in $main_pair);

		next_pair
	}};
}

pub(crate) use {assert_rule, get_pair_safe};

pub(crate) fn pair_into_string(pair: &Pair<'_, Rule>) -> String {
	String::from(pair.as_span().as_str())
}

pub(crate) fn get_bin_operator_from_pair(
	pair: &Pair<'_, Rule>
) -> expressions::operators::BinaryOperator {
	expressions::operators::BinaryOperator::try_from(pair.as_str().to_owned()).unwrap()
}

pub(crate) fn interpret_special_chars(input: &str) -> String {
	let mut result = String::new();
	let mut chars = input.chars();

	while let Some(c) = chars.next() {
		if c != '\\' {
			result.push(c);
			continue;
		}

		let next_char = chars.next();

		if next_char.is_none() {
			result.push('\\');
			continue;
		}

		match next_char.unwrap() {
			'n' => result.push('\n'),
			'r' => result.push('\r'),
			't' => result.push('\t'),
			'"' => result.push('"'),
			'\\' => result.push('\\'),

			// Unknown escape sequence, treat it as a literal
			next_char => {
				result.push('\\');
				result.push(next_char);
			}
		}
	}

	result
}
