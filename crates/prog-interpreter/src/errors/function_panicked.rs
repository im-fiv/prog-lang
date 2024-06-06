use ariadne::{ColorGenerator, Label};
use serde::Serialize;

use prog_utils::pretty_errors::{AriadneCompatible, Span, Position};

#[derive(Debug, Clone, Serialize)]
pub struct FunctionPanicked;

impl AriadneCompatible for FunctionPanicked {
	fn message(&self) -> String {
		String::from("function panicked")
	}

	fn labels(self, file: &str, position: Position) -> Vec<Label<Span>> {
		let mut colors = ColorGenerator::new();
		let color = colors.next();

		vec![
			Label::new((file, position))
				.with_message("during this function call")
				.with_color(color)
		]
	}
}