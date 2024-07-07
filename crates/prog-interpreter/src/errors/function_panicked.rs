use ariadne::{ColorGenerator, Label};
use prog_utils::pretty_errors::{AriadneCompatible, Position, Span};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub struct FunctionPanicked;

impl AriadneCompatible for FunctionPanicked {
	fn message(&self) -> String { String::from("function panicked") }

	fn labels(self, file: &str, position: Position) -> Vec<Label<Span>> {
		let mut colors = ColorGenerator::new();
		let color = colors.next();

		vec![Label::new((file, position))
			.with_message("during this function call")
			.with_color(color)]
	}
}
