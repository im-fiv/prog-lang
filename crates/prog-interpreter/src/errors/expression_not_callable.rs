use ariadne::{ColorGenerator, Label, Fmt};
use serde::Serialize;

use prog_utils::pretty_errors::{AriadneCompatible, Span, Position};
use crate::RuntimeValueKind;

#[derive(Debug, Clone, Serialize)]
pub struct ExpressionNotCallable(
	pub RuntimeValueKind
);

impl AriadneCompatible for ExpressionNotCallable {
	fn message(&self) -> String {
		String::from("expression not callable")
	}

	fn labels(self, file: &str, position: Position) -> Vec<Label<Span>> {
		let mut colors = ColorGenerator::new();

		let color_expected = colors.next();
		let color_got = colors.next();

		vec![
			Label::new((file, position))
				.with_message(format!(
					"expected expression of type {}, got {}",
					"Function".fg(color_expected),
					self.0.to_string().fg(color_got)
				))
				.with_color(color_got)
		]
	}
}