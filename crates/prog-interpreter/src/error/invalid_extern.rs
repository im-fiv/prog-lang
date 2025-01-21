use ariadne::{Fmt, Label};
use prog_utils::pretty_errors::{color_generator, AriadneCompatible, Span};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct InvalidExtern(pub(crate) String);

impl<'s> AriadneCompatible<'s> for InvalidExtern {
	fn message(&self) -> &'static str { "invalid extern item" }

	fn labels(&self, span: Span<'s>) -> Vec<ariadne::Label<Span<'s>>> {
		let mut colors = color_generator();

		let color_item = colors.next();

		vec![Label::new(span)
			.with_message(format!(
				"extern item with name `{}` was not found",
				(&self.0).fg(color_item)
			))
			.with_color(color_item)]
	}
}
