use ariadne::{ColorGenerator, Fmt, Label};
use prog_utils::pretty_errors::{AriadneCompatible, Position, Span};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct NonExhaustiveClassConstruction(pub Vec<String>);

impl AriadneCompatible for NonExhaustiveClassConstruction {
	fn message(&self) -> String { String::from("non-exhaustive class construction") }

	fn labels(self, file: &str, position: Position) -> Vec<Label<Span>> {
		let mut colors = ColorGenerator::new();
		let color = colors.next();

		let fields = self
			.0
			.into_iter()
			.map(|f| format!("`{f}`"))
			.collect::<Vec<_>>();

		vec![Label::new(Span::new(file, position))
			.with_message(format!("missing fields: {}", fields.join(", ").fg(color)))
			.with_color(color)]
	}
}
