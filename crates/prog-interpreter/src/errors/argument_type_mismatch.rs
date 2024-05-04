use ariadne::{ColorGenerator, Label, Fmt};
use serde::Serialize;

use prog_utils::pretty_errors::{AriadneCompatible, Span, Position};

#[derive(Debug, Clone, Serialize)]
pub struct ArgumentTypeMismatch {
	pub expected: String,
	pub got: String,
	pub function_pos: Position
}

impl AriadneCompatible for ArgumentTypeMismatch {
	fn message(&self) -> String {
		String::from("argument type mismatch")
	}

	fn labels(self, file: &str, position: Position) -> Vec<Label<Span>> {
		let mut colors = ColorGenerator::new();

		let color_expected = colors.next();
		let color_got = colors.next();

		vec![
			Label::new((file, position))
				.with_message(format!(
					"expected argument of type `{}`, got `{}`",
					self.expected.fg(color_expected),
					self.got.fg(color_got)
				))
				.with_color(color_got),
			
			Label::new((file, self.function_pos))
				.with_message("function in question")
				.with_color(color_expected)
		]
	}
}