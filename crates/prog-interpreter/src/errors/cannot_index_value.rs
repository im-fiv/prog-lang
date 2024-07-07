use ariadne::{ColorGenerator, Fmt, Label};
use prog_utils::pretty_errors::{AriadneCompatible, Position, Span};

use crate::RuntimeValueKind;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub struct CannotIndexValue {
	pub kind: (RuntimeValueKind, Position),
	pub expected_index_kind: RuntimeValueKind,
	pub index_kind: (RuntimeValueKind, Position),
	pub because_negative: bool
}

impl AriadneCompatible for CannotIndexValue {
	fn message(&self) -> String { String::from("cannot index value") }

	fn labels(self, file: &str, _position: Position) -> Vec<Label<Span>> {
		let mut colors = ColorGenerator::new();

		let color_list = colors.next();
		let color_expected = colors.next();
		let color_got = colors.next();

		let (message_expected, message_got) =
			if self.because_negative && (self.expected_index_kind == self.index_kind.0) {
				let expected = format!(
					"type {} expects an index of type {} and of {} value",
					self.kind.0.to_string().fg(color_list),
					self.expected_index_kind.to_string().fg(color_expected),
					"positive".fg(color_expected)
				);

				let got = format!(
					"got same type, but of {} value instead",
					"negative".fg(color_got)
				);

				(expected, got)
			} else {
				let expected = format!(
					"type {} expects an index of type {}",
					self.kind.0.to_string().fg(color_list),
					self.expected_index_kind.to_string().fg(color_expected)
				);

				let got = format!(
					"got one with type {} instead",
					self.index_kind.0.to_string().fg(color_got)
				);

				(expected, got)
			};

		vec![
			Label::new((file, self.kind.1))
				.with_message(message_expected)
				.with_color(color_list)
				.with_order(0),
			Label::new((file, self.index_kind.1))
				.with_message(message_got)
				.with_color(color_got)
				.with_order(1),
		]
	}
}
