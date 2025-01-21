use ariadne::Label;
use prog_utils::pretty_errors::{AriadneCompatible, Span, color_generator};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Internal(pub(crate) String);

impl<'s> AriadneCompatible<'s> for Internal {
	fn message(&self) -> &'static str { "internal parser error" }

	fn labels(&self, span: Span<'s>) -> Vec<ariadne::Label<Span<'s>>> {
		let mut colors = color_generator();

		vec![Label::new(span)
			.with_message(&self.0)
			.with_color(colors.next())]
	}
}
