mod arg_parser;
mod context;
pub mod error;
mod intrinsics;
mod shared;
pub mod value;

pub use context::{Context, ContextFlags};
pub use error::{InterpretError, InterpretErrorKind};
pub use shared::Shared;
pub(crate) use value::{Callable, CallableData};
pub use value::{Primitive, Value, ValueKind};

use prog_parser::{ast, ASTNode};

pub type InterpretResult<'s, T> = Result<T, InterpretError<'s>>;

pub trait Evaluatable<'ast> {
	type Output;

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<'ast, Self::Output>;
}

#[derive(Debug)]
pub struct Interpreter<'ast> {
	stdin: Vec<u8>,
	stdout: Vec<u8>,

	pub context: Context<'ast>
}

impl<'ast> Interpreter<'ast> {
	pub fn new() -> Self {
		let table = intrinsics::IntrinsicTable::new();
		Self::new_empty().populate(table)
	}

	pub fn new_empty() -> Self {
		Self {
			stdin: vec![],
			stdout: vec![],

			context: Context::new()
		}
	}

	pub fn evaluate<N>(&mut self, node: N) -> InterpretResult<'ast, N::Output>
	where
		N: Evaluatable<'ast>
	{
		node.evaluate(self)
	}

	fn populate(self, table: intrinsics::IntrinsicTable<'ast>) -> Self {
		for intrinsic in table {
			if intrinsic.auto_import {
				let name = intrinsic.name.to_owned();
				let value = intrinsic.value.clone();

				assert!(
					self.context.insert(name, value).is_none(),
					"Attempted to override item `{}` with an intrinsic",
					intrinsic.name
				);
			}

			let name = intrinsic.name.to_owned();
			let value = intrinsic.value.clone();

			assert!(
				self.context.insert_extern(name, value).is_none(),
				"Attempted to override extern item `{}`",
				intrinsic.name
			)
		}

		self
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

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<'ast, Self::Output> {
		let value = self.stmts.evaluate(i)?;

		if let Value::CtrlFlow(value::CtrlFlow::Return(ret)) = value {
			return Ok(*ret);
		}

		Ok(value)
	}
}

impl<'ast> Evaluatable<'ast> for Vec<ast::Stmt<'ast>> {
	type Output = Value<'ast>;

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<'ast, Self::Output> {
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

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<'ast, Self::Output> {
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

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<'ast, Self::Output> {
		match self {
			Self::Binary(expr) => expr.evaluate(i),
			Self::Unary(expr) => expr.evaluate(i),
			Self::Term(expr) => expr.evaluate(i)
		}
	}
}

impl<'ast> Evaluatable<'ast> for ast::BinaryExpr<'ast> {
	type Output = Value<'ast>;

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<'ast, Self::Output> {
		use ast::BinaryOpKind as Op;
		use Value as V;

		let span_expr = self.span();

		let lhs = self.lhs.evaluate(i)?;
		let rhs = self.rhs.evaluate(i)?;

		Ok(match (self.op.kind, lhs, rhs) {
			(Op::Plus, V::Num(lhs), V::Num(rhs)) => V::Num(lhs + rhs),
			(Op::Minus, V::Num(lhs), V::Num(rhs)) => V::Num(lhs - rhs),
			(Op::Asterisk, V::Num(lhs), V::Num(rhs)) => V::Num(lhs * rhs),
			(Op::Slash, V::Num(lhs), V::Num(rhs)) => V::Num(lhs / rhs),
			(Op::Sign, V::Num(lhs), V::Num(rhs)) => V::Num(lhs % rhs),

			(Op::EqEq, lhs, rhs) => V::Bool(value::Bool::from(lhs == rhs)),
			(Op::Neq, lhs, rhs) => V::Bool(value::Bool::from(lhs != rhs)),

			// TODO
			_ => {
				return Err(InterpretError::new(
					span_expr,
					InterpretErrorKind::Unimplemented(error::Unimplemented)
				))
			}
		})
	}
}

impl<'ast> Evaluatable<'ast> for ast::UnaryExpr<'ast> {
	type Output = Value<'ast>;

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<'ast, Self::Output> {
		use ast::UnaryOpKind as Op;
		use Value as V;

		let span_expr = self.span();

		let operand = self.operand.evaluate(i)?;

		Ok(match (self.op.kind, operand) {
			(Op::Minus, V::Num(operand)) => V::Num(-operand),

			(Op::Not, V::Bool(operand)) => V::Bool(!operand),
			(Op::Not, operand) => V::Bool(value::Bool::from(operand.is_truthy())),

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

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<'ast, Self::Output> {
		let span_term = self.span();

		match self {
			Self::Expr(expr) => expr.evaluate(i),
			Self::ParenExpr(expr) => expr.expr.evaluate(i),

			Self::Lit(lit) => lit.evaluate(i),
			Self::Ident(ident) => ident.evaluate(i),
			Self::Func(func) => func.evaluate(i).map(Value::Func),
			Self::List(list) => list.evaluate(i).map(Value::List),
			Self::Obj(obj) => obj.evaluate(i).map(Value::Obj),
			Self::Extern(ext) => ext.evaluate(i),
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

	fn evaluate(self, _: &mut Interpreter<'ast>) -> InterpretResult<'ast, Self::Output> {
		use ast::LitKind;

		Ok(match self.kind {
			LitKind::Num(lit) => Value::Num(value::Num::from(lit)),
			LitKind::Bool(lit) => Value::Bool(value::Bool::from(lit)),
			LitKind::Str(lit) => Value::Str(value::Str::from(lit)),
			LitKind::None => Value::None
		})
	}
}

impl<'ast> Evaluatable<'ast> for ast::Ident<'ast> {
	type Output = Value<'ast>;

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<'ast, Self::Output> {
		let name = self.value();
		let value = i.context.get(name);

		value.ok_or(InterpretError::new(
			self.span(),
			InterpretErrorKind::VarDoesntExist(error::VarDoesntExist(self.value_owned()))
		))
	}
}

impl<'ast> Evaluatable<'ast> for ast::Func<'ast> {
	type Output = value::Func<'ast>;

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<'ast, Self::Output> {
		use arg_parser::{Arg, ArgList};

		let ctx = i.context.child();

		let args = if !self.args.is_empty() {
			let args = self
				.args
				.items()
				.into_iter()
				.map(|a| Arg::RequiredUntyped(Box::from(a.value())))
				.collect::<Vec<_>>();

			ArgList::new(args)
		} else {
			ArgList::new_empty()
		};

		Ok(value::Func {
			ast: Box::new(self),
			args,
			ctx
		})
	}
}

impl<'ast> Evaluatable<'ast> for ast::List<'ast> {
	type Output = value::List<'ast>;

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<'ast, Self::Output> {
		let items = self
			.items
			.unwrap_items()
			.into_iter()
			.map(|item| item.evaluate(i))
			.collect::<InterpretResult<Vec<_>>>()?;

		Ok(value::List::new(items))
	}
}

impl<'ast> Evaluatable<'ast> for ast::Obj<'ast> {
	type Output = value::Obj<'ast>;

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<'ast, Self::Output> {
		use std::collections::hash_map::{Entry, HashMap};

		let mut entry_map = HashMap::new();

		for entry in self.fields.unwrap_items().into_iter() {
			let name = entry.name.value_owned();
			let value = entry.value.evaluate(i)?;

			match entry_map.entry(name) {
				Entry::Vacant(e) => {
					e.insert((value, entry.name.span()));
				}

				Entry::Occupied(e) => {
					let def_name = e.get().1;

					return Err(InterpretError::new(
						entry.name.span(),
						InterpretErrorKind::DuplicateObjEntry(error::DuplicateObjEntry {
							def_name
						})
					));
				}
			}
		}

		// Stripping position info as it is no longer needed
		let entry_map = entry_map
			.into_iter()
			.map(|(key, (value, _))| (key, value))
			.collect::<HashMap<_, _>>();

		Ok(value::Obj::from(entry_map))
	}
}

impl<'ast> Evaluatable<'ast> for ast::Extern<'ast> {
	type Output = Value<'ast>;

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<'ast, Self::Output> {
		let span_value = self.value.span();
		let value = match self.value.evaluate(i)? {
			Value::Str(s) => <value::Str as Into<String>>::into(s),

			v => {
				return Err(InterpretError::new(
					span_value,
					InterpretErrorKind::ArgTypeMismatch(error::ArgTypeMismatch {
						expected: ValueKind::Str,
						found: v.kind()
					})
				))
			}
		};

		i.context.get_extern(&value).ok_or(InterpretError::new(
			span_value,
			InterpretErrorKind::InvalidExtern(error::InvalidExtern(value))
		))
	}
}

// `Call` is considered an expression term
impl<'ast> Evaluatable<'ast> for ast::Call<'ast> {
	type Output = Value<'ast>;

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<'ast, Self::Output> {
		use arg_parser::ArgumentParseError;
		use prog_parser::{Position, Span};

		let span_callee = self.callee.span();
		let span_args = {
			// Parentheses are included in the span in case the argument list is empty
			let source = self.source();
			let file = self.file();
			let position = Position::new(self._lp.start(), self._rp.end());

			Span::new(source, file, position)
		};

		let call_site = value::CallSite {
			callee: span_callee,
			_lp: self._lp.span(),
			args: self.args.map_ref(ASTNode::span, ASTNode::span),
			_rp: self._rp.span()
		};

		let func = match self.callee.evaluate(i)? {
			Value::Func(f) => Box::new(f) as Box<dyn Callable>,
			Value::IntrinsicFn(f) => Box::new(f) as Box<dyn Callable>,

			v => {
				return Err(InterpretError::new(
					span_callee,
					InterpretErrorKind::ExprNotCallable(error::ExprNotCallable(v.kind()))
				));
			}
		};

		// Is there really no way to write this better?
		let (arg_spans, arg_values) = self
			.args
			.unwrap_items()
			.into_iter()
			.map(|e| (e.span(), e.evaluate(i)))
			.unzip::<_, _, Vec<_>, Vec<_>>();
		let arg_values = arg_values
			.into_iter()
			.collect::<InterpretResult<Vec<_>>>()?;

		let parsed_args = func.arg_list().verify(&arg_values).map_err(|e| {
			match e {
				ArgumentParseError::CountMismatch {
					expected,
					end_boundary,
					found
				} => {
					InterpretError::new(
						span_args,
						InterpretErrorKind::ArgCountMismatch(error::ArgCountMismatch {
							expected,
							end_boundary,
							found
						})
					)
				}

				ArgumentParseError::IncorrectType {
					index,
					expected,
					found
				} => {
					let arg_span = arg_spans.get(index).copied().unwrap();

					InterpretError::new(
						arg_span,
						InterpretErrorKind::ArgTypeMismatch(error::ArgTypeMismatch {
							expected,
							found
						})
					)
				}
			}
		})?;

		func.call(CallableData {
			i,
			args: parsed_args,
			call_site
		})
	}
}

//* Statements *//

impl<'ast> Evaluatable<'ast> for ast::VarDefine<'ast> {
	type Output = Value<'ast>;

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<'ast, Self::Output> {
		let name = self.name().value();
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

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<'ast, Self::Output> {
		let span_name = self.name.span();

		let name = self.name.value_owned();
		let value = self.value.evaluate(i)?;

		if i.context.update(&name, value).is_none() {
			return Err(InterpretError::new(
				span_name,
				InterpretErrorKind::VarDoesntExist(error::VarDoesntExist(name))
			));
		}

		Ok(Value::None)
	}
}

impl<'ast> Evaluatable<'ast> for ast::DoBlock<'ast> {
	type Output = Value<'ast>;

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<'ast, Self::Output> {
		let original_ctx = i.context.swap(i.context.child());
		let result = self.stmts.evaluate(i);
		i.context.swap(original_ctx);

		result
	}
}

impl<'ast> Evaluatable<'ast> for ast::Return<'ast> {
	type Output = Value<'ast>;

	fn evaluate(self, i: &mut Interpreter<'ast>) -> InterpretResult<'ast, Self::Output> {
		let value = self.value.evaluate(i).map(Box::new)?;
		Ok(Value::CtrlFlow(value::CtrlFlow::Return(value)))
	}
}
