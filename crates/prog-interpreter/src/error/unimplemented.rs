use ariadne::Label;
use prog_utils::pretty_errors::{color_generator, AriadneCompatible, Span};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Unimplemented;

impl<'s> AriadneCompatible<'s> for Unimplemented {
	fn message(&self) -> &'static str { "unimplemented" }

	fn labels(&self, span: Span<'s>) -> Vec<ariadne::Label<Span<'s>>> {
		let mut colors = color_generator();

		vec![Label::new(span).with_color(colors.next())]
	}
}
