use ariadne::{ColorGenerator, Fmt, Label};
use prog_utils::pretty_errors::{AriadneCompatible, Position, Span};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct NonExistentExternItem(pub String);

impl AriadneCompatible for NonExistentExternItem {
	fn message(&self) -> String { String::from("non-existent extern item") }

	fn labels(self, file: &str, position: Position) -> Vec<Label<Span>> {
		let mut colors = ColorGenerator::new();
		let color = colors.next();

		vec![Label::new(Span::new_unchecked(file, position))
			.with_message(format!(
				"Extern item with name `{}` was not found",
				self.0.fg(color)
			))
			.with_color(color)]
	}
}
