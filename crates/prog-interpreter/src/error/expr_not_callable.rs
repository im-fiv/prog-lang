use ariadne::{ColorGenerator, Fmt, Label};
use prog_utils::pretty_errors::{AriadneCompatible, Span};

use crate::ValueKind;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprNotCallable(pub ValueKind);

impl AriadneCompatible for ExprNotCallable {
	fn message(&self) -> String { String::from("expression is not callable") }

	fn labels(self, span: Span) -> Vec<Label<Span>> {
		let mut colors = ColorGenerator::new();

		let color_expected = colors.next();
		let color_got = colors.next();

		vec![Label::new(span)
			.with_message(format!(
				"expected expression of type {}, got {}",
				ValueKind::Func.fg(color_expected),
				self.0.to_string().fg(color_got)
			))
			.with_color(color_got)]
	}
}
