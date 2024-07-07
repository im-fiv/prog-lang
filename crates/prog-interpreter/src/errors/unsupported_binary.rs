use ariadne::{ColorGenerator, Fmt, Label};
use prog_parser::ast::expressions::operators::BinaryOperator;
use prog_utils::pretty_errors::{AriadneCompatible, Position, Span};

use crate::RuntimeValueKind;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub struct UnsupportedBinary {
	pub lhs: (RuntimeValueKind, Position),
	pub operator: (BinaryOperator, Position),
	pub rhs: (RuntimeValueKind, Position)
}

impl AriadneCompatible for UnsupportedBinary {
	fn message(&self) -> String { String::from("unsupported operation") }

	fn labels(self, file: &str, _position: Position) -> Vec<Label<Span>> {
		let mut colors = ColorGenerator::new();

		let color_operator = colors.next();
		let color_operands = colors.next();

		vec![
			Label::new((file, self.operator.1))
				.with_message("this operation")
				.with_color(color_operator)
				.with_order(0),
			Label::new((file, self.lhs.1.start..self.rhs.1.end))
				.with_message(format!(
					"cannot be performed on types {} and {}",
					self.lhs.0.to_string().fg(color_operands),
					self.rhs.0.to_string().fg(color_operands)
				))
				.with_color(color_operands)
				.with_order(1),
		]
	}
}
