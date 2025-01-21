use ariadne::{Fmt, Label};
use prog_utils::pretty_errors::{AriadneCompatible, Span, color_generator};

use crate::ValueKind;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprNotCallable(pub(crate) ValueKind);

impl<'s> AriadneCompatible<'s> for ExprNotCallable {
	fn message(&self) -> &'static str { "expression is not callable" }

	fn labels(&self, span: Span<'s>) -> Vec<Label<Span<'s>>> {
		let mut colors = color_generator();

		let color_expected = colors.next();
		let color_found = colors.next();

		vec![Label::new(span)
			.with_message(format!(
				"expected expression of type `{}`, found `{}`",
				ValueKind::Func.fg(color_expected),
				self.0.fg(color_found)
			))
			.with_color(color_found)]
	}
}
