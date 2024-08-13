use std::fmt::{self, Debug, Display};

use prog_parser::ast;

//* Note: `Debug` is implemented manually below
#[derive(Clone, PartialEq)]
pub struct RuntimeFunction {
	pub ast: Box<ast::expressions::Function>,
	pub source: String,
	pub file: String
}

impl Debug for RuntimeFunction {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let arguments = self
			.ast
			.arguments
			.iter()
			.map(|(name, _)| name)
			.cloned()
			.collect::<Vec<_>>();

		f.debug_struct("Function")
			.field("arguments", &arguments)
			.field("statements", &self.ast.statements)
			.field("file", &self.file)
			.finish()
	}
}

impl Display for RuntimeFunction {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
