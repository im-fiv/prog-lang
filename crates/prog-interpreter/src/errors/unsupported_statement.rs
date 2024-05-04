use ariadne::{ColorGenerator, Label, Fmt};
use serde::Serialize;

use prog_utils::pretty_errors::{AriadneCompatible, Span, Position};

#[derive(Debug, Clone, Serialize)]
pub struct UnsupportedStatement {
	pub statement: String,
	pub position: Position
}

impl AriadneCompatible for UnsupportedStatement {
	fn message(&self) -> String {
		String::from("unsupported statement")
	}

	fn labels(self, file: &str) -> Vec<Label<Span>> {
		let mut colors = ColorGenerator::new();
		let color = colors.next();

		vec![
			Label::new((file, self.position))
				.with_message(format!(
					"`{}` is unsupported",
					self.statement.fg(color)
				))
				.with_color(color)
		]
	}
}