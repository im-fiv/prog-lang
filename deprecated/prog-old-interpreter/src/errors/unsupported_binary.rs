use ariadne::{ColorGenerator, Fmt, Label};
use prog_parser::ast::BinaryOpKind;
use prog_utils::pretty_errors::{AriadneCompatible, Position, Span};

use crate::ValueKind;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct UnsupportedBinary {
	pub lhs: (ValueKind, Position),
	pub operator: (BinaryOpKind, Position),
	pub rhs: (ValueKind, Position)
}

impl AriadneCompatible for UnsupportedBinary {
	fn message(&self) -> String { String::from("unsupported operation") }

	fn labels(self, file: &str, _position: Position) -> Vec<Label<Span>> {
		let mut labels = vec![];
		let mut colors = ColorGenerator::new();

		let color_operator = colors.next();
		let color_operands = colors.next();

		labels.push(
			Label::new(Span::new_unchecked(file, self.operator.1))
				.with_message("this operation")
				.with_color(color_operator)
				.with_order(0)
		);

		let position = Position::new(self.lhs.1.start(), self.rhs.1.end());
		labels.push(
			Label::new(Span::new_unchecked(file, position))
				.with_message(format!(
					"cannot be performed on types {} and {}",
					self.lhs.0.to_string().fg(color_operands),
					self.rhs.0.to_string().fg(color_operands)
				))
				.with_color(color_operands)
				.with_order(1)
		);

		labels
	}
}
