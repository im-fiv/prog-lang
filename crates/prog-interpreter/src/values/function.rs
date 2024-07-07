use std::fmt::Display;

use prog_parser::ast;

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeFunction {
	pub ast: Box<ast::expressions::Function>,
	pub source: String,
	pub file: String
}

impl Display for RuntimeFunction {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let arguments_str = self
			.ast
			.arguments
			.iter()
			.map(|(a, _)| a.to_owned())
			.collect::<Vec<_>>()
			.join(", ");

		let formatted = format!("Function({arguments_str})");
		write!(f, "{formatted}")
	}
}
