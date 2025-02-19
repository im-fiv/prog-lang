use ariadne::{Fmt, Label};

use prog_utils::pretty_errors::{color_generator, AriadneCompatible, Span};
use prog_utils::JoinWith;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum InvalidClassConstruction {
	UnknownField {
		class_name: String,
		field_name: String
	},

	MissingFields {
		class_name: String,
		field_names: Vec<String>
	}
}

impl<'s> AriadneCompatible<'s> for InvalidClassConstruction {
	fn message(&self) -> &'static str { "invalid class construction" }

	fn labels(&self, span: Span<'s>) -> Vec<ariadne::Label<Span<'s>>> {
		let mut colors = color_generator();

		match self {
			Self::UnknownField {
				class_name,
				field_name
			} => {
				let color_field = colors.next();
				let color_class = colors.next();

				vec![Label::new(span)
					.with_message(format!(
						"class `{}` does not have a field named `{}`",
						class_name.fg(color_class),
						field_name.fg(color_field)
					))
					.with_color(color_field)]
			}

			Self::MissingFields {
				class_name,
				field_names
			} => {
				let color_fields = colors.next();
				let color_class = colors.next();

				vec![Label::new(span)
					.with_message(format!(
						"missing fields {} for class `{}`",
						field_names.fmt_join_with(|f| format!("`{}`", f.fg(color_fields)), "and"),
						class_name.fg(color_class)
					))
					.with_color(color_fields)]
			}
		}
	}
}
