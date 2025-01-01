use ariadne::{ColorGenerator, Fmt, Label};
use prog_utils::pretty_errors::{AriadneCompatible, Position, Span};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FieldDoesntExist(pub String, pub Position);

impl AriadneCompatible for FieldDoesntExist {
	fn message(&self) -> String { String::from("field doesn't exist") }

	fn labels(self, file: &str, position: Position) -> Vec<Label<Span>> {
		let mut colors = ColorGenerator::new();

		let color_field = colors.next();
		let color_object = colors.next();

		vec![
			Label::new(Span::new(file, self.1))
				.with_message(format!(
					"field `{}` does not exist on",
					self.0.fg(color_field)
				))
				.with_color(color_field)
				.with_order(0),
			Label::new(Span::new(file, position))
				.with_message(String::from("this object"))
				.with_color(color_object)
				.with_order(1),
		]
	}
}
