use ariadne::{ColorGenerator, Label};
use prog_utils::pretty_errors::{AriadneCompatible, Span};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Internal(pub String);

impl<'s> AriadneCompatible<'s> for Internal {
	fn message(&self) -> String { String::from("internal error") }

	fn labels(&self, span: Span<'s>) -> Vec<ariadne::Label<Span<'s>>> {
		let mut colors = ColorGenerator::new();

		vec![Label::new(span)
			.with_message(&self.0)
			.with_color(colors.next())]
	}
}
