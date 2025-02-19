use ariadne::{Fmt, Label};
use prog_utils::pretty_errors::{color_generator, AriadneCompatible, Span};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CtxDisallowed {
	/// Name of the thing that is not allowed in current context.
	pub(crate) thing: String,
	/// Whether the name of the thing is plural.
	pub(crate) plural: bool
}

impl<'s> AriadneCompatible<'s> for CtxDisallowed {
	fn message(&self) -> &'static str { "context disallowed" }

	fn labels(&self, span: Span<'s>) -> Vec<ariadne::Label<Span<'s>>> {
		let mut colors = color_generator();

		let color_thing = colors.next();

		vec![Label::new(span)
			.with_message(format!(
				"{} in current context {} not allowed",
				(&self.thing).fg(color_thing),
				if self.plural { "are" } else { "is" }
			))
			.with_color(color_thing)]
	}
}
