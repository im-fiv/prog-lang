use ariadne::{ColorGenerator, Label};
use prog_utils::pretty_errors::{AriadneCompatible, Span};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Unimplemented;

impl AriadneCompatible for Unimplemented {
	fn message(&self) -> String { String::from("unimplemented") }

	fn labels(self, span: Span) -> Vec<ariadne::Label<Span>> {
		let mut colors = ColorGenerator::new();

		vec![Label::new(span).with_color(colors.next())]
	}
}
