use ariadne::{ColorGenerator, Label, Fmt};
use serde::Serialize;
use std::ops::Range;

use prog_utils::pretty_errors::{AriadneCompatible, Span, Position};

#[derive(Debug, Clone, Serialize)]
pub struct ArgumentCountMismatch {
	pub expected: Range<usize>,
	pub end_boundary: bool,
	pub got: usize,
	pub function_pos: Position
}

impl AriadneCompatible for ArgumentCountMismatch {
	fn message(&self) -> String {
		String::from("argument count mismatch")
	}

	fn labels(self, file: &str, position: Position) -> Vec<Label<Span>> {
		let mut colors = ColorGenerator::new();

		let color_expected = colors.next();
		let color_got = colors.next();

		let message = {
			if self.expected.start == self.expected.end && self.end_boundary {
				format!(
					"expected {} arguments, got {}",
					self.expected.start.fg(color_expected),
					self.got.fg(color_got)
				)
			} else if self.expected.start != self.expected.end && self.end_boundary {
				format!(
					"expected {} arguments at least and {} at most, got {}",
					self.expected.start.fg(color_expected),
					self.expected.end.fg(color_expected),
					self.got.fg(color_got)
				)
			} else if self.expected.start == self.expected.end && !self.end_boundary {
				format!(
					"expected at least {} arguments, got {}",
					self.expected.start.fg(color_expected),
					self.got.fg(color_got)
				)
			} else {
				unreachable!("Something went wrong!")
			}
		};

		vec![
			Label::new((file, position))
				.with_message(message)
				.with_color(color_got),
			
			Label::new((file, self.function_pos))
				.with_message("function in question")
				.with_color(color_expected)
		]
	}
}