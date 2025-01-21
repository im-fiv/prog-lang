use ariadne::{Label, Fmt};
use prog_utils::pretty_errors::{AriadneCompatible, Span, color_generator};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct DuplicateObjEntry<'s> {
	/// Span of the originally defined entry's name.
	pub(crate) def_name: Span<'s>
}

impl<'s> AriadneCompatible<'s> for DuplicateObjEntry<'s> {
	fn message(&self) -> &'static str { "duplicate object entry" }

	fn labels(&self, span: Span<'s>) -> Vec<ariadne::Label<Span<'s>>> {
		let mut colors = color_generator();

		let color_def = colors.next();
		let color_redef = colors.next();

		vec![
			Label::new(self.def_name)
				.with_message(format!(
					"redefinition of {} field...",
					"this".fg(color_def)
				))
				.with_color(color_def),

			Label::new(span)
				.with_message(format!(
					"...occurs {}",
					"here".fg(color_redef)
				))
				.with_color(color_redef)
		]
	}
}
