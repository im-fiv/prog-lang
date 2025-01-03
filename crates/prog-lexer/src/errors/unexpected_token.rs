use ariadne::{ColorGenerator, Label};
use prog_utils::pretty_errors::{AriadneCompatible, Position, Span};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct UnexpectedToken {
	pub got: char,
	pub expected: Option<char>
}

impl AriadneCompatible for UnexpectedToken {
	fn message(&self) -> String { String::from("unexpected token") }

	fn labels(self, file: &str, position: Position) -> Vec<ariadne::Label<Span>> {
		let mut colors = ColorGenerator::new();

		let mut label = Label::new(Span::new_unchecked(file, position))
			.with_message(&format!("unexpected `{}`", self.got))
			.with_color(colors.next());

		if let Some(expected) = self.expected {
			label = label.with_message(&format!("expected `{}`, got `{}`", expected, self.got));
		}

		vec![label]
	}
}