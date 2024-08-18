use ariadne::{ColorGenerator, Fmt, Label};
use prog_parser::ast::expressions::operators::UnaryOperator;
use prog_utils::pretty_errors::{AriadneCompatible, Position, Span};

use crate::ValueKind;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct UnsupportedUnary {
	pub operator: (UnaryOperator, Position),
	pub operand: (ValueKind, Position)
}

impl AriadneCompatible for UnsupportedUnary {
	fn message(&self) -> String { String::from("unsupported operation") }

	fn labels(self, file: &str, _position: Position) -> Vec<Label<Span>> {
		let mut colors = ColorGenerator::new();

		let color_operator = colors.next();
		let color_operand = colors.next();

		vec![
			Label::new((file, self.operator.1))
				.with_message("this operation")
				.with_color(color_operator)
				.with_order(0),
			Label::new((file, self.operand.1))
				.with_message(format!(
					"cannot be performed on type {}",
					self.operand.0.to_string().fg(color_operand)
				))
				.with_color(color_operand)
				.with_order(1),
		]
	}
}
