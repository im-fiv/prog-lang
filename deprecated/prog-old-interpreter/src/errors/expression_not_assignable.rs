use ariadne::{ColorGenerator, Fmt, Label};
use prog_utils::pretty_errors::{AriadneCompatible, Position, Span};

use crate::ValueKind;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExpressionNotAssignable(pub Option<ValueKind>);

impl AriadneCompatible for ExpressionNotAssignable {
	fn message(&self) -> String { String::from("expression cannot be assigned to") }

	fn labels(self, file: &str, position: Position) -> Vec<Label<Span>> {
		let mut colors = ColorGenerator::new();

		let color_expected = colors.next();
		let color_got = colors.next();

		let message = if let Some(kind) = self.0 {
			format!(
				"expected expression of type {}, {} or {}, got {}",
				"List".fg(color_expected),
				"Object".fg(color_expected),
				"ClassInstance".fg(color_expected),
				kind.to_string().fg(color_got)
			)
		} else {
			format!(
				"expected expression of type {}, {} or {}",
				"List".fg(color_expected),
				"Object".fg(color_expected),
				"ClassInstance".fg(color_expected)
			)
		};

		vec![Label::new(Span::new_unchecked(file, position))
			.with_message(message)
			.with_color(color_got)]
	}
}
