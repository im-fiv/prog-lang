use ariadne::{ColorGenerator, Label};
use prog_utils::pretty_errors::{AriadneCompatible, Position, Span};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct UnexpectedToken(pub char);

impl AriadneCompatible for UnexpectedToken {
	fn message(&self) -> String { String::from("unexpected token") }

	fn labels(self, file: &str, position: Position) -> Vec<ariadne::Label<Span>> {
		let mut colors = ColorGenerator::new();

		vec![Label::new(Span::new(file, position))
			.with_message(&format!("unexpected `{}`", self.0))
			.with_color(colors.next())]
	}
}
