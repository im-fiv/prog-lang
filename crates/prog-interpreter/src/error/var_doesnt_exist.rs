use ariadne::{Fmt, Label};
use prog_utils::pretty_errors::{AriadneCompatible, Span, color_generator};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct VarDoesntExist(pub String);

impl<'s> AriadneCompatible<'s> for VarDoesntExist {
	fn message(&self) -> &'static str { "variable doesn't exist" }

	fn labels(&self, span: Span<'s>) -> Vec<Label<Span<'s>>> {
		let mut colors = color_generator();
		let color = colors.next();

		vec![Label::new(span)
			.with_message(format!("`{}` hasn't yet been defined", (&self.0).fg(color)))
			.with_color(color)]
	}
}
