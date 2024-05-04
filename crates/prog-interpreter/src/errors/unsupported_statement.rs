use ariadne::{ColorGenerator, Label, Fmt};
use serde::Serialize;

use prog_utils::pretty_errors::{AriadneCompatible, Span, Position};

#[derive(Debug, Clone, Serialize)]
pub struct UnsupportedStatement(
	pub String
);

impl AriadneCompatible for UnsupportedStatement {
	fn message(&self) -> String {
		String::from("unsupported statement")
	}

	fn labels(self, file: &str, position: Position) -> Vec<Label<Span>> {
		let mut colors = ColorGenerator::new();
		let color = colors.next();

		vec![
			Label::new((file, position))
				.with_message(format!(
					"`{}` is unsupported",
					self.0.fg(color)
				))
				.with_color(color)
		]
	}
}