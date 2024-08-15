use std::fmt::{self, Debug, Display};

use halloc::HeapMutator;
use prog_parser::ast;

use crate::context::RuntimeContext;

//* Note: `Debug` and `PartialEq` are implemented manually below
#[derive(Clone)]
pub struct RuntimeFunction {
	pub ast: Box<ast::expressions::Function>,

	pub source: String,
	pub file: String,

	pub context: HeapMutator<'static, RuntimeContext>
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

impl PartialEq for RuntimeFunction {
	fn eq(&self, other: &Self) -> bool {
		let ast = self.ast == other.ast;
		let source = self.source == other.source;
		let file = self.file == other.file;

		ast && source && file
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

		let formatted = format!("func({arguments_str})");
		write!(f, "{formatted}")
	}
}
