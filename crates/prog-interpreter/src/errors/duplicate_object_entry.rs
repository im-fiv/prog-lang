use ariadne::{ColorGenerator, Label, Fmt};
use prog_utils::pretty_errors::{AriadneCompatible, Span, Position};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub struct DuplicateObjectEntry {
	pub entry_name: String,
	pub definition_pos: Position
}

impl AriadneCompatible for DuplicateObjectEntry {
	fn message(&self) -> String {
		String::from("duplicate object entry")
	}

	fn labels(self, file: &str, position: Position) -> Vec<Label<Span>> {
		let mut colors = ColorGenerator::new();

		let color_entry = colors.next();
		let color_definiton = colors.next();

		vec![
			Label::new((file, position))
				.with_message(format!(
					"object entry `{}`",
					self.entry_name.fg(color_entry)
				))
				.with_color(color_entry)
				.with_order(0),
			
			Label::new((file, self.definition_pos))
				.with_message(format!(
					"has already been defined {}",
					"here".fg(color_definiton)
				))
				.with_color(color_definiton)
				.with_order(1)
		]
	}
}