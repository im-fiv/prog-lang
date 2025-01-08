use ariadne::{ColorGenerator, Label};
use prog_utils::pretty_errors::{AriadneCompatible, Span};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Internal(pub String);

impl AriadneCompatible for Internal {
	fn message(&self) -> String { String::from("internal error") }

	fn labels(self, span: Span) -> Vec<ariadne::Label<Span>> {
		let mut colors = ColorGenerator::new();

		vec![Label::new(span)
			.with_message(self.0)
			.with_color(colors.next())]
	}
}
