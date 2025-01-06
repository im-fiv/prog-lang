use std::fmt::{self, Debug, Display};

use prog_parser::{ast, ASTNode};

use crate::context::Context;

#[derive(Clone)]
pub struct RFunction {
	pub ast: Box<ast::Func<'static>>,

	pub source: String,
	pub file: String,

	pub context: Context
}

impl PartialEq for RFunction {
	fn eq(&self, other: &Self) -> bool {
		let ast = self.ast == other.ast;
		let source = self.source == other.source;
		let file = self.file == other.file;

		ast && source && file
	}
}

impl Debug for RFunction {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let args = self
			.ast
			.args
			.as_ref()
			.map(|p| {
				p.items()
					.into_iter()
					.map(|i| i.value_owned())
					.collect::<Vec<_>>()
			})
			.unwrap_or_default();

		f.debug_struct("Function")
			.field("arguments", &args)
			.field("statements", &self.ast.stmts)
			.field("file", &self.file)
			.finish()
	}
}

impl Display for RFunction {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let args = self
			.ast
			.args
			.as_ref()
			.map(|p| {
				p.items()
					.into_iter()
					.map(|i| i.value_owned())
					.collect::<Vec<_>>()
			})
			.unwrap_or_default()
			.join(", ");

		write!(f, "func({args})")
	}
}
