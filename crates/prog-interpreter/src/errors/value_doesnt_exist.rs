use ariadne::{ColorGenerator, Fmt, Label};
use prog_utils::pretty_errors::{AriadneCompatible, Position, Span};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ValueDoesntExist(pub String);

impl AriadneCompatible for ValueDoesntExist {
	fn message(&self) -> String { String::from("value doesn't exist") }

	fn labels(self, file: &str, position: Position) -> Vec<Label<Span>> {
		let mut colors = ColorGenerator::new();
		let color = colors.next();

		vec![Label::new((file, position))
			.with_message(format!("`{}` hasn't yet been defined", self.0.fg(color)))
			.with_color(color)]
	}
}
