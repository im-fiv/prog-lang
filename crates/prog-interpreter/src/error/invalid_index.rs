use ariadne::{Fmt, Label};
use prog_utils::pretty_errors::{color_generator, AriadneCompatible, Span};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct InvalidIndex<'s>(pub(crate) crate::Value<'s>);

impl<'s> AriadneCompatible<'s> for InvalidIndex<'s> {
	fn message(&self) -> &'static str { "invalid list index" }

	fn labels(&self, span: Span<'s>) -> Vec<ariadne::Label<Span<'s>>> {
		let mut colors = color_generator();

		let color_expected = colors.next();
		let color_found = colors.next();

		vec![Label::new(span)
			.with_message(format!(
				"expected a {}, found `{}`",
				"positive whole number".fg(color_expected),
				(&self.0).fg(color_found)
			))
			.with_color(colors.next())]
	}
}
