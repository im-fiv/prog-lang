use ariadne::{ColorGenerator, Fmt, Label};
use prog_utils::pretty_errors::{AriadneCompatible, Position, Span};

use crate::ValueKind;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExpressionNotCallable(pub ValueKind);

impl AriadneCompatible for ExpressionNotCallable {
	fn message(&self) -> String { String::from("expression is not callable") }

	fn labels(self, file: &str, position: Position) -> Vec<Label<Span>> {
		let mut colors = ColorGenerator::new();

		let color_expected = colors.next();
		let color_got = colors.next();

		vec![Label::new(Span::new(file, position))
			.with_message(format!(
				"expected expression of type {}, got {}",
				"Function".fg(color_expected),
				self.0.to_string().fg(color_got)
			))
			.with_color(color_got)]
	}
}
