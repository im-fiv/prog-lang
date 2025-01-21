use ariadne::Label;
use prog_utils::pretty_errors::{AriadneCompatible, Span, color_generator};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AssertionFailed(pub Option<String>);

impl<'s> AriadneCompatible<'s> for AssertionFailed {
	fn message(&self) -> &'static str { "assertion failed" }

	fn labels(&self, span: Span<'s>) -> Vec<Label<Span<'s>>> {
		let mut colors = color_generator();

		let mut label = Label::new(span).with_color(colors.next());

		if let Some(ref message) = self.0 {
			label = label.with_message(message);
		}

		vec![label]
	}
}
