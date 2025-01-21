use ariadne::{Fmt, Label};

use prog_utils::pretty_errors::{color_generator, AriadneCompatible, Span};
use prog_utils::JoinWithOr;

use crate::ValueKind;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CannotIndexExpr {
	pub expected: Vec<ValueKind>,
	pub found: ValueKind
}

impl<'s> AriadneCompatible<'s> for CannotIndexExpr {
	fn message(&self) -> &'static str { "cannot index expression" }

	fn labels(&self, span: Span<'s>) -> Vec<ariadne::Label<Span<'s>>> {
		let mut colors = color_generator();

		let color_expected = colors.next();
		let color_found = colors.next();

		vec![Label::new(span)
			.with_message(format!(
				"expected an expression of type {}, found `{}`",
				self.expected
					.fmt_join_with(|ty| format!("`{}`", ty.fg(color_expected))),
				self.found.fg(color_found)
			))
			.with_color(color_found)]
	}
}
