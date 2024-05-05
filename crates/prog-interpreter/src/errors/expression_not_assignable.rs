use ariadne::{ColorGenerator, Label, Fmt};
use serde::Serialize;

use prog_utils::pretty_errors::{AriadneCompatible, Span, Position};
use crate::RuntimeValueKind;

#[derive(Debug, Clone, Serialize)]
pub struct ExpressionNotAssignable(
	pub Option<RuntimeValueKind>
);

impl AriadneCompatible for ExpressionNotAssignable {
	fn message(&self) -> String {
		String::from("expression cannot be assigned to")
	}

	fn labels(self, file: &str, position: Position) -> Vec<Label<Span>> {
		let mut colors = ColorGenerator::new();

		let color_expected = colors.next();
		let color_got = colors.next();

		let message = if let Some(kind) = self.0 {
			format!(
				"expected expression of type {} or {}, got {}",
				"List".fg(color_expected),
				"Object".fg(color_expected),
				kind.to_string().fg(color_got)
			)
		} else {
			format!(
				"expected expression of type {} or {}",
				"List".fg(color_expected),
				"Object".fg(color_expected)
			)
		};

		vec![
			Label::new((file, position))
				.with_message(message)
				.with_color(color_got)
		]
	}
}