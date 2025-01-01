use std::ops::Range;

use ariadne::{ColorGenerator, Fmt, Label};
use prog_utils::pretty_errors::{AriadneCompatible, Position, Span};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ArgumentCountMismatch {
	pub expected: Range<usize>,
	pub end_boundary: bool,
	pub got: usize,
	pub fn_call_pos: Position,
	pub fn_def_args_pos: Option<Position>
}

impl AriadneCompatible for ArgumentCountMismatch {
	fn message(&self) -> String { String::from("argument count mismatch") }

	fn labels(self, file: &str, position: Position) -> Vec<Label<Span>> {
		let mut colors = ColorGenerator::new();

		let color_expected = colors.next();
		let color_got = colors.next();

		let message = {
			if self.expected.start == self.expected.end && self.end_boundary {
				format!(
					"expected {} argument{}, got {}",
					self.expected.start.fg(color_expected),
					if self.expected.start != 1 { "s" } else { "" },
					self.got.fg(color_got)
				)
			} else if self.expected.start != self.expected.end && self.end_boundary {
				format!(
					"expected {} argument{} at least and {} at most, got {}",
					self.expected.start.fg(color_expected),
					if self.expected.start != 1 { "s" } else { "" },
					self.expected.end.fg(color_expected),
					self.got.fg(color_got)
				)
			} else if self.expected.start == self.expected.end && !self.end_boundary {
				format!(
					"expected at least {} argument{}, got {}",
					self.expected.start.fg(color_expected),
					if self.expected.start != 1 { "s" } else { "" },
					self.got.fg(color_got)
				)
			} else {
				unreachable!("Something went wrong!")
			}
		};

		let mut labels = vec![
			Label::new(Span::new(file, position))
				.with_message(message)
				.with_color(color_got),
			Label::new(Span::new(file, self.fn_call_pos)).with_color(color_expected),
		];

		if let Some(fn_def_args_pos) = self.fn_def_args_pos {
			let definition_label = Label::new(Span::new(file, fn_def_args_pos))
				.with_message(format!("as defined {}", "here".fg(color_expected)))
				.with_color(color_expected);

			labels.push(definition_label);
		}

		labels
	}
}
