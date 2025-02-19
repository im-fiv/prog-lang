use ariadne::{Fmt, Label};
use prog_utils::pretty_errors::{color_generator, AriadneCompatible, Span};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ClassFnReassign {
	pub(crate) class_name: String,
	pub(crate) field_name: String
}

impl<'s> AriadneCompatible<'s> for ClassFnReassign {
	fn message(&self) -> &'static str { "cannot reassign class functions" }

	fn labels(&self, span: Span<'s>) -> Vec<ariadne::Label<Span<'s>>> {
		let mut colors = color_generator();

		let color_field = colors.next();
		let color_class = colors.next();

		vec![Label::new(span)
			.with_message(format!(
				"field `{}` of class `{}` is a function and cannot be reassigned",
				(&self.field_name).fg(color_field),
				(&self.class_name).fg(color_class)
			))
			.with_color(color_field)]
	}
}
