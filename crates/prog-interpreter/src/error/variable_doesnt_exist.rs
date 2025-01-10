use ariadne::{ColorGenerator, Fmt, Label};
use prog_utils::pretty_errors::{AriadneCompatible, Span};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct VariableDoesntExist(pub String);

impl AriadneCompatible for VariableDoesntExist {
	fn message(&self) -> String { String::from("value doesn't exist") }

	fn labels(self, span: Span) -> Vec<Label<Span>> {
		let mut colors = ColorGenerator::new();
		let color = colors.next();

		vec![Label::new(span)
			.with_message(format!("`{}` hasn't yet been defined", self.0.fg(color)))
			.with_color(color)]
	}
}
