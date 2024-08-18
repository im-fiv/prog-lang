use ariadne::{ColorGenerator, Label};
use prog_utils::pretty_errors::{AriadneCompatible, Position, Span};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CannotReassignClassFunctions;

impl AriadneCompatible for CannotReassignClassFunctions {
	fn message(&self) -> String { String::from("expression cannot be assigned to") }

	fn labels(self, file: &str, position: Position) -> Vec<Label<Span>> {
		let mut colors = ColorGenerator::new();
		let color = colors.next();

		vec![Label::new((file, position))
			.with_message("class function cannot be reassigned")
			.with_color(color)]
	}
}
