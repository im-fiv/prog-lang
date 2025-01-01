use ariadne::{ColorGenerator, Label};
use prog_utils::pretty_errors::{AriadneCompatible, Position, Span};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct InvalidFile(pub String);

impl AriadneCompatible for InvalidFile {
	fn message(&self) -> String { String::from("invalid file") }

	fn labels(self, file: &str, position: Position) -> Vec<Label<Span>> {
		let mut colors = ColorGenerator::new();

		vec![Label::new(Span::new_unchecked(file, position))
			.with_message("file with specified path does not exist or is invalid")
			.with_color(colors.next())]
	}
}
