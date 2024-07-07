use ariadne::{ColorGenerator, Label};
use prog_utils::pretty_errors::{AriadneCompatible, Position, Span};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub struct AssertionFailed(pub Option<String>);

impl AriadneCompatible for AssertionFailed {
	fn message(&self) -> String { String::from("assertion failed") }

	fn labels(self, file: &str, position: Position) -> Vec<Label<Span>> {
		let mut colors = ColorGenerator::new();

		let mut label = Label::new((file, position)).with_color(colors.next());

		if let Some(message) = self.0 {
			label = label.with_message(format!("\"{message}\""));
		}

		vec![label]
	}
}
