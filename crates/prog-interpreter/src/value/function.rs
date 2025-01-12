use std::fmt::{self, Display};

use prog_parser::{ast, token, ASTNode};

use crate::{Context, Evaluatable, InterpretResult, Interpreter, Primitive, Value};

#[derive(Debug, Clone)]
pub struct Func<'ast> {
	pub ast: Box<ast::Func<'ast>>,
	pub ctx: Context<'ast>
}

impl<'ast> Func<'ast> {
	pub fn args(&self) -> Vec<&str> {
		self.ast
			.args
			.as_ref()
			.map(|punct| {
				punct
					.items()
					.into_iter()
					.map(|arg| arg.value())
					.collect::<Vec<_>>()
			})
			.unwrap_or_default()
	}

	pub fn call(
		self,
		args: Option<ast::Punctuated<'ast, ast::Expr<'ast>, token::Comma<'ast>>>,
		i: &mut Interpreter<'ast>
	) -> InterpretResult<'ast, Value<'ast>> {
		let args_expected = self.ast.args.unwrap_or_default().unwrap_items();

		let args_got = args
			.unwrap_or_default()
			.unwrap_items()
			.into_iter()
			.map(|arg| arg.evaluate(i))
			.collect::<InterpretResult<Vec<_>>>()?;

		if args_got.len() != args_expected.len() {
			// TODO: argument checking
			todo!()
		}

		for (arg_name, arg_value) in args_expected.into_iter().zip(args_got) {
			self.ctx.insert(arg_name.value_owned(), arg_value);
		}

		let original_ctx = i.context.swap(self.ctx);
		// Unlike a vector of statements, a function must produce a final value
		let result = ast::Program {
			stmts: self.ast.stmts
		}
		.evaluate(i);
		i.context.swap(original_ctx);

		result
	}
}

impl Primitive for Func<'_> {
	fn is_truthy(&self) -> bool { true }
}

impl PartialEq for Func<'_> {
	fn eq(&self, other: &Self) -> bool { self.ast == other.ast }
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

impl Display for Func<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let func = &self.ast._func;
		let lp = &self.ast._lp;
		let args = self.args().join(", ");
		let rp = &self.ast._rp;

		write!(f, "{func}{lp}{args}{rp}")
	}
}
