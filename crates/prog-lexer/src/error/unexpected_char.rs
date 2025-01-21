use ariadne::{Label, Fmt};
use prog_utils::pretty_errors::{AriadneCompatible, Span, color_generator};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct UnexpectedChar {
	pub(crate) found: char,
	pub(crate) expected: Option<char>
}

impl<'s> AriadneCompatible<'s> for UnexpectedChar {
	fn message(&self) -> &'static str { "unexpected token" }

	fn labels(&self, span: Span<'s>) -> Vec<ariadne::Label<Span<'s>>> {
		let mut colors = color_generator();

		let color_expected = colors.next();
		let color_found = colors.next();

		let mut label = Label::new(span)
			.with_message(format!(
				"unexpected `{}`",
				self.found.fg(color_found)
			))
			.with_color(color_found);

		if let Some(expected) = self.expected {
			label = label.with_message(format!(
				"expected `{}`, found `{}`",
				expected.fg(color_expected),
				self.found.fg(color_found)
			));
		}

		vec![label]
	}
}
