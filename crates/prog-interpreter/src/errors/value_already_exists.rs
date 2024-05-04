use ariadne::{ColorGenerator, Label, Fmt};
use serde::Serialize;

use prog_utils::pretty_errors::{AriadneCompatible, Span, Position};

#[derive(Debug, Clone, Serialize)]
pub struct ValueAlreadyExists {
	pub value: String,
	pub position: Position
}

impl AriadneCompatible for ValueAlreadyExists {
	fn message(&self) -> String {
		String::from("value already exists")
	}

	fn labels(self, file: &str) -> Vec<Label<Span>> {
		let mut colors = ColorGenerator::new();
		let color = colors.next();

		vec![
			Label::new((file, self.position))
				.with_message(format!(
					"`{}` has already been defined",
					self.value.fg(color)
				))
				.with_color(color)
		]
	}
}