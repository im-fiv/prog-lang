use ariadne::{Fmt, Label};

use prog_utils::pretty_errors::{color_generator, AriadneCompatible, Span};
use prog_utils::JoinWith;

use crate::ValueKind;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprNotCallable {
	pub(crate) expected: Vec<ValueKind>,
	pub(crate) found: ValueKind
}

impl<'s> AriadneCompatible<'s> for ExprNotCallable {
	fn message(&self) -> &'static str { "expression is not callable" }

	fn labels(&self, span: Span<'s>) -> Vec<Label<Span<'s>>> {
		let mut colors = color_generator();

		let color_expected = colors.next();
		let color_found = colors.next();

		vec![Label::new(span)
			.with_message(format!(
				"expected expression of type {}, found `{}`",
				self.expected
					.fmt_join_with(|e| format!("`{}`", e.fg(color_expected)), "or"),
				self.found.fg(color_found)
			))
			.with_color(color_found)]
	}
}
