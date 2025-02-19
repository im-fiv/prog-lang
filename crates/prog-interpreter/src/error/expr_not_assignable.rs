use ariadne::{Fmt, Label};
use prog_utils::pretty_errors::{color_generator, AriadneCompatible, Span};
use prog_utils::JoinWith;

use crate::ValueKind;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprNotAssignable {
	pub(crate) expected: Vec<ValueKind>,
	pub(crate) found: ValueKind
}

impl<'s> AriadneCompatible<'s> for ExprNotAssignable {
	fn message(&self) -> &'static str { "expression cannot be assigned to" }

	fn labels(&self, span: Span<'s>) -> Vec<Label<Span<'s>>> {
		let mut colors = color_generator();

		let color_found = colors.next();
		let color_expected = colors.next();

		vec![Label::new(span)
			.with_message(format!(
				"expected an expression of type {}, found `{}`",
				self.expected
					.fmt_join_with(|ty| format!("`{}`", ty.fg(color_expected)), "or"),
				self.found.fg(color_found)
			))
			.with_color(color_found)]
	}
}
