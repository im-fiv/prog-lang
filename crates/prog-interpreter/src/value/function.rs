use std::borrow::Cow;
use std::fmt::{self, Debug, Display};
use std::rc::Rc;

use prog_parser::{ast, ASTNode};

use crate::arg_parser::{ArgList, ParsedArg};
use crate::{Callable, CallableData, Context, Evaluatable, InterpretResult, Primitive, Value};

#[derive(Clone)]
pub struct Func<'ast> {
	pub(crate) ast: Rc<ast::Func<'ast>>,
	pub(crate) args: ArgList,
	pub(crate) ctx: Context<'ast>
}

impl Func<'_> {
	pub(crate) fn args_str(&self) -> Vec<&str> {
		match &self.ast.args {
			ast::FuncArgs::WithSelf { _self, args } => {
				let args = args
					.as_ref()
					.map(|(_, args)| args.map_ref_items(|arg| arg.value()).unwrap_items())
					.unwrap_or_default();

				let mut result = vec![];
				result.push(_self.value());
				result.extend(args);
				result
			}

			ast::FuncArgs::WithoutSelf { args } => {
				args.map_ref_items(|arg| arg.value()).unwrap_items()
			}
		}
	}
}

impl Primitive for Func<'_> {
	fn is_truthy(&self) -> bool { true }
}

impl<'intref, 'int: 'intref> Callable<'intref, 'int> for Func<'int> {
	fn arg_list(&self) -> Cow<ArgList> { Cow::Borrowed(&self.args) }

	fn call(
		&mut self,
		CallableData { i, mut args, .. }: CallableData<'intref, 'int>
	) -> InterpretResult<'int, Value<'int>> {
		for (name, value) in args.drain() {
			let ParsedArg::Regular(value) = value else {
				panic!("Runtime function received variadic arguments");
			};

			self.ctx.insert(name, value);
		}

		Context::swap_in_place(&mut i.context, &mut self.ctx);
		// Unlike a vector of statements, a function must produce a final value
		let stmts = ast::Program {
			stmts: Rc::clone(&self.ast.block.stmts)
		};

		let result = stmts.evaluate(i);
		Context::swap_in_place(&mut i.context, &mut self.ctx);

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

impl Debug for Func<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut s = f.debug_struct("Func");
		s.field("ast", &self.ast);
		s.field("args", &self.args);
		s.finish()
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
		s.serialize_field("ast", &*self.ast)?;
		s.end()
	}
}
