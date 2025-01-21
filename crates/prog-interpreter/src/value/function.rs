use std::fmt::{self, Display};

use prog_parser::{ast, ASTNode};

use crate::{Context, Evaluatable, InterpretResult, Primitive, Callable, CallableData, Value};
use crate::arg_parser::{ArgList, ParsedArg};

#[derive(Debug, Clone)]
pub struct Func<'ast> {
	pub(crate) ast: Box<ast::Func<'ast>>,
	pub(crate) args: ArgList,
	pub(crate) ctx: Context<'ast>
}

impl<'ast> Func<'ast> {
	pub(crate) fn args_str(&self) -> Vec<&str> {
		self.ast
			.args
			.items()
			.into_iter()
			.map(|arg| arg.value())
			.collect::<Vec<_>>()
	}
}

impl Primitive for Func<'_> {
	fn is_truthy(&self) -> bool { true }
}

impl<'intref, 'int: 'intref> Callable<'intref, 'int> for Func<'int> {
	fn arg_list(&self) -> &ArgList { &self.args }

	fn call(
		self: Box<Self>,
		CallableData {
			i,
			mut args,
			..
		}: CallableData<'intref, 'int>
	) -> InterpretResult<'int, Value<'int>> {
		for (name, value) in args.drain() {
			let ParsedArg::Regular(value) = value else {
				panic!("Runtime function received variadic arguments");
			};

			self.ctx.insert(name, value);
		}

		let original_ctx = i.context.swap(self.ctx);
		// Unlike a vector of statements, a function must produce a final value
		let stmts = ast::Program { stmts: self.ast.stmts };
		let result = stmts.evaluate(i);
		i.context.swap(original_ctx);

		result
	}
}

impl PartialEq for Func<'_> {
	fn eq(&self, other: &Self) -> bool { self.ast == other.ast }
}

impl Display for Func<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let func = &self.ast._func;
		let lp = &self.ast._lp;
		let args = self.args_str().join(", ");
		let rp = &self.ast._rp;

		write!(f, "{func}{lp}{args}{rp}")
	}
}

#[cfg(feature = "serde")]
impl serde::Serialize for Func<'_> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer
	{
		use serde::ser::SerializeStruct;

		let mut s = serializer.serialize_struct("Func", 1)?;
		s.serialize_field("ast", &self.ast)?;
		s.end()
	}
}
