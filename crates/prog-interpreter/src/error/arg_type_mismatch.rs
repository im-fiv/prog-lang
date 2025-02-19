use ariadne::{Fmt, Label};
use prog_utils::pretty_errors::{color_generator, AriadneCompatible, Span};

use crate::ValueKind;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ArgTypeMismatch {
	pub(crate) expected: ValueKind,
	pub(crate) found: ValueKind
}

impl<'s> AriadneCompatible<'s> for ArgTypeMismatch {
	fn message(&self) -> &'static str { "argument type mismatch" }

	fn labels(&self, span: Span<'s>) -> Vec<ariadne::Label<Span<'s>>> {
		let mut colors = color_generator();

		let color_expected = colors.next();
		let color_found = colors.next();

		vec![Label::new(span)
			.with_message(format!(
				"expected an argument of type `{}`, found `{}`",
				self.expected.fg(color_expected),
				self.found.fg(color_found)
			))
			.with_color(colors.next())]
	}
}
