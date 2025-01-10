mod context;
pub mod error;
mod shared;
pub mod value;

pub use context::{Context, ContextFlags};
pub use error::{InterpretError, InterpretErrorKind};
pub use shared::Shared;
pub use value::{Primitive, Value, ValueKind};

use prog_parser::{ast, ASTNode};

pub type InterpretResult<T> = Result<T, InterpretError>;

pub trait Evaluatable<'ast> {
	type Output;

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<Self::Output>;
}

#[derive(Debug)]
pub struct Interpreter<'ast> {
	stdin: Vec<u8>,
	stdout: Vec<u8>,

	pub context: Context<'ast>
}

impl<'ast> Interpreter<'ast> {
	pub fn new() -> Self {
		Self {
			stdin: vec![],
			stdout: vec![],

			context: Context::new()
		}
	}

	pub fn evaluate<N>(&mut self, node: N) -> InterpretResult<N::Output>
	where
		N: Evaluatable<'ast>
	{
		node.evaluate(self)
	}

	pub fn stdin(&self) -> &[u8] { &self.stdin }

	pub fn stdin_mut(&mut self) -> &mut [u8] { &mut self.stdin }

	pub fn stdout(&self) -> &[u8] { &self.stdout }

	pub fn stdout_mut(&mut self) -> &mut [u8] { &mut self.stdout }
}

impl Default for Interpreter<'_> {
	fn default() -> Self { Self::new() }
}

//* `Program` and `Stmt` *//

// The `Evaluatable` implementations for `ast::Program` and `Vec<ast::Stmt>` differ in how they handle return values.
//
// A `Program` represents a complete unit of execution intended to yield a final value.
// Its `evaluate` method explicitly checks for and extracts a returned value from a return statement.
//
// A `Vec<ast::Stmt>`, on the other hand, executes statements sequentially for their side effects (e.g., variable assignments, printing)
// and potential control flow changes (e.g., early exits with `return`).
// It doesn't itself produce a final result beyond the execution of those side effects.
impl<'ast> Evaluatable<'ast> for ast::Program<'ast> {
	type Output = Value<'ast>;

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<Self::Output> {
		let value = self.stmts.evaluate(i)?;

		if let Value::CtrlFlow(value::CtrlFlow::Return(ret)) = value {
			return Ok(*ret);
		}

		Ok(value)
	}
}

impl<'ast> Evaluatable<'ast> for Vec<ast::Stmt<'ast>> {
	type Output = Value<'ast>;

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<Self::Output> {
		for stmt in self {
			let value = stmt.evaluate(i)?;

			if value.kind() == ValueKind::CtrlFlow {
				return Ok(value);
			}
		}

		Ok(Value::None)
	}
}

impl<'ast> Evaluatable<'ast> for ast::Stmt<'ast> {
	type Output = Value<'ast>;

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<Self::Output> {
		match self {
			Self::VarDefine(stmt) => stmt.evaluate(i),
			Self::VarAssign(stmt) => stmt.evaluate(i),
			Self::DoBlock(stmt) => stmt.evaluate(i),
			Self::Return(stmt) => stmt.evaluate(i),
			Self::Call(stmt) => stmt.evaluate(i),

			// TODO
			stmt => {
				Err(InterpretError::new(
					stmt.span(),
					InterpretErrorKind::Unimplemented(error::Unimplemented)
				))
			}
		}
	}
}

//* Expressions *//

impl<'ast> Evaluatable<'ast> for ast::Expr<'ast> {
	type Output = Value<'ast>;

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<Self::Output> {
		match self {
			Self::Binary(expr) => expr.evaluate(i),
			Self::Unary(expr) => expr.evaluate(i),
			Self::Term(expr) => expr.evaluate(i)
		}
	}
}

impl<'ast> Evaluatable<'ast> for ast::BinaryExpr<'ast> {
	type Output = Value<'ast>;

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<Self::Output> {
		use ast::BinaryOpKind as Op;
		use Value as V;

		let span_expr = self.span();

		let lhs = self.lhs.evaluate(i)?;
		let rhs = self.rhs.evaluate(i)?;

		match (self.op.kind, lhs, rhs) {
			(Op::Plus, V::Num(lhs), V::Num(rhs)) => Ok(V::Num(lhs + rhs)),
			(Op::Minus, V::Num(lhs), V::Num(rhs)) => Ok(V::Num(lhs - rhs)),
			(Op::Asterisk, V::Num(lhs), V::Num(rhs)) => Ok(V::Num(lhs * rhs)),
			(Op::Slash, V::Num(lhs), V::Num(rhs)) => Ok(V::Num(lhs / rhs)),
			(Op::Sign, V::Num(lhs), V::Num(rhs)) => Ok(V::Num(lhs % rhs)),

			// TODO
			_ => {
				Err(InterpretError::new(
					span_expr,
					InterpretErrorKind::Unimplemented(error::Unimplemented)
				))
			}
		}
	}
}

impl<'ast> Evaluatable<'ast> for ast::UnaryExpr<'ast> {
	type Output = Value<'ast>;

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<Self::Output> {
		use ast::UnaryOpKind as Op;
		use Value as V;

		let span_expr = self.span();

		let operand = self.operand.evaluate(i)?;

		Ok(match (self.op.kind, operand) {
			(Op::Minus, V::Num(operand)) => V::Num(-operand),

			(Op::Not, V::Bool(operand)) => V::Bool(!operand),
			(Op::Not, operand) => V::Bool(value::Bool::new(operand.is_truthy())),

			_ => {
				return Err(InterpretError::new(
					span_expr,
					InterpretErrorKind::Unimplemented(error::Unimplemented)
				))
			}
		})
	}
}

impl<'ast> Evaluatable<'ast> for ast::Term<'ast> {
	type Output = Value<'ast>;

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<Self::Output> {
		let span_term = self.span();

		match self {
			Self::Expr(expr) => expr.evaluate(i),
			Self::ParenExpr(expr) => expr.expr.evaluate(i),

			Self::Lit(lit) => lit.evaluate(i),
			Self::Ident(ident) => ident.evaluate(i),
			Self::Func(func) => func.evaluate(i).map(Value::Func),
			// TODO: list
			// TODO: object
			// TODO: extern
			Self::Call(call) => call.evaluate(i),

			// TODO
			_ => {
				Err(InterpretError::new(
					span_term,
					InterpretErrorKind::Unimplemented(error::Unimplemented)
				))
			}
		}
	}
}

impl<'ast> Evaluatable<'ast> for ast::Lit<'ast> {
	type Output = Value<'ast>;

	fn evaluate(self, _: &mut Interpreter<'ast>) -> InterpretResult<Self::Output> {
		use ast::LitKind;

		Ok(match self.kind {
			LitKind::Num(lit) => Value::Num(value::Num::new(lit)),
			LitKind::Bool(lit) => Value::Bool(value::Bool::new(lit)),
			LitKind::Str(lit) => Value::Str(value::Str::new(lit)),
			LitKind::None => Value::None
		})
	}
}

impl<'ast> Evaluatable<'ast> for ast::Ident<'ast> {
	type Output = Value<'ast>;

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<Self::Output> {
		let name = self.value();
		let value = i.context.get(name);

		value.ok_or(InterpretError::new(
			self.span(),
			InterpretErrorKind::VariableDoesntExist(error::VariableDoesntExist(self.value_owned()))
		))
	}
}

impl<'ast> Evaluatable<'ast> for ast::Func<'ast> {
	type Output = value::Func<'ast>;

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<Self::Output> {
		let ctx = i.context.child();

		Ok(value::Func {
			ast: Box::new(self),
			ctx
		})
	}
}

// `Call` is considered an expression term
impl<'ast> Evaluatable<'ast> for ast::Call<'ast> {
	type Output = Value<'ast>;

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<Self::Output> {
		let span_func = self.func.span();

		let func = match self.func.evaluate(i)? {
			Value::Func(f) => f,

			val => {
				return Err(InterpretError::new(
					span_func,
					InterpretErrorKind::ExpressionNotCallable(error::ExpressionNotCallable(
						val.kind()
					))
				));
			}
		};

		let args = self.args.map(|p| *p);

		func.call(args, i)
	}
}

//* Statements *//

impl<'ast> Evaluatable<'ast> for ast::VarDefine<'ast> {
	type Output = Value<'ast>;

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<Self::Output> {
		let name = self.name().value_owned();
		let value = match self.value() {
			Some(val) => val.evaluate(i)?,
			None => Value::None
		};

		i.context.insert(name, value);
		Ok(Value::None)
	}
}

impl<'ast> Evaluatable<'ast> for ast::VarAssign<'ast> {
	type Output = Value<'ast>;

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<Self::Output> {
		let span_name = self.name.span();

		let name = self.name.value_owned();
		let value = self.value.evaluate(i)?;

		if i.context.update(name.clone(), value).is_none() {
			return Err(InterpretError::new(
				span_name,
				InterpretErrorKind::VariableDoesntExist(error::VariableDoesntExist(name))
			));
		}

		Ok(Value::None)
	}
}

impl<'ast> Evaluatable<'ast> for ast::DoBlock<'ast> {
	type Output = Value<'ast>;

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<Self::Output> {
		let original_ctx = i.context.swap(i.context.child());
		let result = self.stmts.evaluate(i);
		i.context.swap(original_ctx);

		result
	}
}

impl<'ast> Evaluatable<'ast> for ast::Return<'ast> {
	type Output = Value<'ast>;

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<Self::Output> {
		let value = self.value.evaluate(i).map(Box::new)?;
		Ok(Value::CtrlFlow(value::CtrlFlow::Return(value)))
	}
}
