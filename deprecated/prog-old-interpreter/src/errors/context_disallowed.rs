use ariadne::{ColorGenerator, Fmt, Label};
use prog_utils::pretty_errors::{AriadneCompatible, Position, Span};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ContextDisallowed {
	pub thing: String,
	pub plural: bool
}

impl AriadneCompatible for ContextDisallowed {
	fn message(&self) -> String { String::from("context disallowed") }

	fn labels(self, file: &str, position: Position) -> Vec<Label<Span>> {
		let mut colors = ColorGenerator::new();
		let color = colors.next();

		vec![Label::new(Span::new_unchecked(file, position))
			.with_message(format!(
				"{} in this context {} not allowed",
				self.thing.fg(color),
				if self.plural { "are" } else { "is" }
			))
			.with_color(color)]
	}
}
