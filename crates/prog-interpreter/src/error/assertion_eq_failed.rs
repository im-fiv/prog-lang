use ariadne::{Fmt, Label};
use prog_utils::pretty_errors::{color_generator, AriadneCompatible, Span};

use crate::Value;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AssertionEqFailed<'s> {
	pub(crate) left: (Value<'s>, Span<'s>),
	pub(crate) right: (Value<'s>, Span<'s>)
}

impl<'s> AriadneCompatible<'s> for AssertionEqFailed<'s> {
	fn message(&self) -> &'static str { "assertion failed" }

	fn labels(&self, _span: Span<'s>) -> Vec<Label<Span<'s>>> {
		let mut colors = color_generator();

		let color_left = colors.next();
		let color_right = colors.next();

		vec![
			Label::new(self.left.1)
				.with_message(format!("(left) `{:#}`", (&self.left.0).fg(color_left)))
				.with_color(color_left),
			Label::new(self.right.1)
				.with_message(format!("(right) `{:#}`", (&self.right.0).fg(color_right)))
				.with_color(color_right),
		]
	}
}
