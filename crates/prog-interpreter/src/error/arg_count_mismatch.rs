use std::ops::Range;

use ariadne::{Fmt, Label};
use prog_utils::pretty_errors::{color_generator, AriadneCompatible, Span};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ArgCountMismatch {
	pub(crate) expected: Range<usize>,
	pub(crate) end_boundary: bool,
	pub(crate) found: usize
}

impl<'s> AriadneCompatible<'s> for ArgCountMismatch {
	fn message(&self) -> &'static str { "argument count mismatch" }

	fn labels(&self, span: Span<'s>) -> Vec<Label<Span<'s>>> {
		let mut colors = color_generator();

		let color_expected = colors.next();
		let color_found = colors.next();

		let message = {
			if self.expected.start == self.expected.end && self.end_boundary {
				format!(
					"expected {} argument{}, found {}",
					self.expected.start.fg(color_expected),
					if self.expected.start != 1 { "s" } else { "" },
					self.found.fg(color_found)
				)
			} else if self.expected.start != self.expected.end && self.end_boundary {
				format!(
					"expected {} argument{} at least and {} at most, found {}",
					self.expected.start.fg(color_expected),
					if self.expected.start != 1 { "s" } else { "" },
					self.expected.end.fg(color_expected),
					self.found.fg(color_found)
				)
			} else if self.expected.start == self.expected.end && !self.end_boundary {
				format!(
					"expected at least {} argument{}, found {}",
					self.expected.start.fg(color_expected),
					if self.expected.start != 1 { "s" } else { "" },
					self.found.fg(color_found)
				)
			} else {
				unreachable!("Something went wrong!")
			}
		};

		vec![Label::new(span)
			.with_message(message)
			.with_color(color_found)]
	}
}
