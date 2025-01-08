use ariadne::{ColorGenerator, Fmt, Label};
use prog_utils::pretty_errors::{AriadneCompatible, Position, Span};

use crate::ValueKind;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct InvalidExternArgument(pub ValueKind);

impl AriadneCompatible for InvalidExternArgument {
	fn message(&self) -> String { String::from("invalid extern argument") }

	fn labels(self, file: &str, position: Position) -> Vec<Label<Span>> {
		let mut colors = ColorGenerator::new();

		let color_expected = colors.next();
		let color_got = colors.next();

		vec![Label::new(Span::new_unchecked(file, position))
			.with_message(format!(
				"Expected value of type `{}`, got `{}`",
				ValueKind::String.fg(color_expected),
				self.0.fg(color_got)
			))
			.with_color(color_got)]
	}
}
