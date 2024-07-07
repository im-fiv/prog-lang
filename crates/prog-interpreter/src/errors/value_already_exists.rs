use ariadne::{ColorGenerator, Fmt, Label};
use prog_utils::pretty_errors::{AriadneCompatible, Position, Span};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub struct ValueAlreadyExists(pub String);

impl AriadneCompatible for ValueAlreadyExists {
	fn message(&self) -> String { String::from("value already exists") }

	fn labels(self, file: &str, position: Position) -> Vec<Label<Span>> {
		let mut colors = ColorGenerator::new();
		let color = colors.next();

		vec![Label::new((file, position))
			.with_message(format!("`{}` has already been defined", self.0.fg(color)))
			.with_color(color)]
	}
}
