use ariadne::{Fmt, Label};
use prog_utils::pretty_errors::{color_generator, AriadneCompatible, Span};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FieldDoesntExist {
	pub(crate) class_name: String,
	pub(crate) field_name: String
}

impl<'s> AriadneCompatible<'s> for FieldDoesntExist {
	fn message(&self) -> &'static str { "field doesn't exist" }

	fn labels(&self, span: Span<'s>) -> Vec<Label<Span<'s>>> {
		let mut colors = color_generator();

		let color_field = colors.next();
		let color_class = colors.next();

		vec![Label::new(span)
			.with_message(format!(
				"class `{}` does not have a field `{}`",
				(&self.class_name).fg(color_class),
				(&self.field_name).fg(color_field)
			))
			.with_color(color_field)]
	}
}
