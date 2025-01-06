pub mod errors;
pub mod values;
pub mod arg_parser;
pub mod context;
pub mod intrinsics;

use std::collections::HashMap;

use anyhow::Result;
use halloc::Memory;
use prog_parser::{ast, ASTNode as _};

use context::Context;
pub use errors::{InterpretError, InterpretErrorKind};
pub use values::{CallSite, RFunction, RPrimitive, Value, ValueKind};

const META_NO_SELF_OVERRIDE: &str = "$NO_SELF_OVERRIDE";
const META_SELF: &str = "self";

/// Only to be used inside the interpreter impl
macro_rules! create_error {
	($self:expr, $position:expr, $kind:expr) => {
		anyhow::bail!(errors::InterpretError::new(
			$self.source.clone(),
			$self.file.clone(),
			$position,
			$kind
		))
	};

	($self:expr, $position:expr, $kind:expr; no_bail) => {
		anyhow::anyhow!(errors::InterpretError::new(
			$self.source.clone(),
			$self.file.clone(),
			$position,
			$kind
		))
	};
}

fn identifier_from_term(term: &ast::Term) -> Option<String> {
	match term {
		ast::Term::Ident(ident) => Some(ident.value().to_owned()),
		ast::Term::Expr(value) => {
			if let ast::Expr::Term(value) = value.as_ref() {
				identifier_from_term(value)
			} else {
				None
			}
		}

		_ => None
	}
}

#[derive(Debug)]
pub struct Interpreter {
	pub memory: Memory,
	pub externs: HashMap<String, Value>,
	pub context: Context,

	source: String,
	file: String
}

impl Interpreter {
	pub fn new() -> Self {
		let mut this = Self::new_clean();

		for (name, (item, bring_into_scope)) in intrinsics::create_variable_table() {
			if bring_into_scope {
				this.context.insert(name.clone(), item.clone());
			}

			this.externs.insert(name, item);
		}

		this
	}

	pub fn new_clean() -> Self {
		Self {
			memory: Memory::new(),
			externs: HashMap::new(),
			context: Context::new(),

			source: String::new(),
			file: String::new()
		}
	}

	pub fn interpret<S, F>(
		&mut self,
		source: S,
		file: F,
		ast: ast::Program<'static>,
		keep_marker: bool
	) -> Result<Value>
	where
		S: Into<String>,
		F: Into<String>
	{
		self.source = source.into();
		self.file = file.into();

		let result = self.execute(ast, keep_marker);

		self.source = String::new();
		self.file = String::new();

		result
	}

	pub fn execute(&mut self, ast: ast::Program<'static>, keep_marker: bool) -> Result<Value> {
		for statement in ast.stmts {
			let result = self.execute_statement(statement)?;

			// In case of `return`, `break`, and `continue` statements
			if let Value::ControlFlow(ref ctrl) = result {
				if keep_marker {
					return Ok(result);
				}

				if let values::ControlFlow::Return(value) = ctrl {
					return Ok(*value.to_owned());
				}

				return Ok(Value::Empty);
			}
		}

		Ok(Value::Empty)
	}

	pub fn execute_statement(&mut self, statement: ast::Statement<'static>) -> Result<Value> {
		match statement {
			ast::Statement::VarDefine(stmt) => self.execute_var_def(stmt),
			ast::Statement::VarAssign(stmt) => self.execute_var_assign(stmt),
			ast::Statement::DoBlock(stmt) => self.execute_do_block(stmt),
			ast::Statement::Return(stmt) => self.execute_return(stmt),
			ast::Statement::Call(stmt) => self.evaluate_call(stmt),
			ast::Statement::WhileLoop(stmt) => self.execute_while_loop(stmt),
			ast::Statement::Break(stmt) => self.execute_break(stmt),
			ast::Statement::Continue(stmt) => self.execute_continue(stmt),
			ast::Statement::If(stmt) => self.execute_if(stmt),
			ast::Statement::ExprAssign(stmt) => self.execute_expr_assign(stmt),
			ast::Statement::ClassDef(stmt) => self.execute_class_def(stmt)
		}
	}

	fn execute_var_def(&mut self, stmt: ast::VarDefine<'static>) -> Result<Value> {
		let name = stmt.name().value_owned();
		let value = match stmt.value() {
			Some(e) => self.evaluate_expr(e, false)?,
			None => Value::Empty
		};

		self.context.insert(name, value);
		Ok(Value::Empty)
	}

	fn execute_var_assign(&mut self, stmt: ast::VarAssign<'static>) -> Result<Value> {
		let value = self.evaluate_expr(stmt.value, false)?;
		let update_result = self.context.update(stmt.name.value_owned(), value);

		if update_result.is_err() {
			create_error!(
				self,
				stmt.name.position(),
				InterpretErrorKind::VariableDoesntExist(errors::VariableDoesntExist(
					stmt.name.value_owned()
				))
			);
		}

		Ok(Value::Empty)
	}

	fn execute_do_block(&mut self, stmt: ast::DoBlock<'static>) -> Result<Value> {
		let stmts = stmt.stmts;

		self.context.deeper();
		let result = self.execute(ast::Program { stmts }, false);
		self.context.shallower();

		result
	}

	fn execute_return(&mut self, stmt: ast::Return<'static>) -> Result<Value> {
		let value = self.evaluate_expr(stmt.value, false)?;

		Ok(Value::ControlFlow(values::ControlFlow::Return(Box::new(
			value
		))))
	}

	fn execute_while_loop(&mut self, stmt: ast::WhileLoop<'static>) -> Result<Value> {
		let mut evaluated = self.evaluate_expr(stmt.cond.clone(), false)?;

		while evaluated.is_truthy() {
			let stmts = stmt.block.stmts.clone();

			self.context.deeper();
			let result = self.execute(ast::Program { stmts }, true)?;
			self.context.shallower();

			if let Value::ControlFlow(ref ctrl) = result {
				match ctrl {
					values::ControlFlow::Return(_) => return Ok(result),
					values::ControlFlow::Break => break,
					values::ControlFlow::Continue => {
						evaluated = self.evaluate_expr(stmt.cond.clone(), false)?;
						continue;
					}
				};
			}

			evaluated = self.evaluate_expr(stmt.cond.clone(), false)?;
		}

		Ok(Value::Empty)
	}

	fn execute_break(&mut self, _stmt: ast::Break) -> Result<Value> {
		Ok(Value::ControlFlow(values::ControlFlow::Break))
	}

	fn execute_continue(&mut self, _stmt: ast::Continue) -> Result<Value> {
		Ok(Value::ControlFlow(values::ControlFlow::Continue))
	}

	fn execute_if(&mut self, stmt: ast::If<'static>) -> Result<Value> {
		let evaluated = self.evaluate_expr(stmt.cond, false)?;

		if evaluated.is_truthy() {
			let stmts = stmt.stmts;

			self.context.deeper();
			let result = self.execute(ast::Program { stmts }, true)?;
			self.context.shallower();

			if result.kind() == ValueKind::ControlFlow {
				return Ok(result);
			}

			return Ok(Value::Empty);
		}

		for branch in stmt.b_elifs {
			let evaluated = self.evaluate_expr(branch.cond, false)?;

			if evaluated.is_truthy() {
				let stmts = branch.stmts;

				self.context.deeper();
				let result = self.execute(ast::Program { stmts }, true)?;
				self.context.shallower();

				if result.kind() == ValueKind::ControlFlow {
					return Ok(result);
				}

				return Ok(Value::Empty);
			}
		}

		if let Some(branch) = stmt.b_else {
			let stmts = branch.stmts;

			self.context.deeper();
			let result = self.execute(ast::Program { stmts }, true)?;
			self.context.shallower();

			if result.kind() == ValueKind::ControlFlow {
				return Ok(result);
			}

			return Ok(Value::Empty);
		}

		Ok(Value::Empty)
	}

	fn execute_expr_assign(&mut self, stmt: ast::ExprAssign<'static>) -> Result<Value> {
		use ast::BinaryOpKind as Op;

		let expr = match stmt.expr {
			ast::Expr::Binary(e) => e,
			e => {
				create_error!(
					self,
					e.position(),
					InterpretErrorKind::ExpressionNotAssignable(errors::ExpressionNotAssignable(
						None
					))
				)
			}
		};

		if !matches!(expr.op.kind, Op::LeftBracket | Op::Dot) {
			create_error!(
				self,
				expr.position(),
				InterpretErrorKind::ExpressionNotAssignable(errors::ExpressionNotAssignable(None))
			);
		}

		let value = self.evaluate_expr(stmt.value, false)?;

		if expr.op.kind == Op::LeftBracket {
			self.execute_expr_assign_list(expr, value)
		} else {
			self.execute_expr_assign_obj(expr, value)
		}
	}

	fn execute_expr_assign_list(
		&mut self,
		expr: ast::BinaryExpr<'static>,
		value: Value
	) -> Result<Value> {
		let list_name = match self.evaluate_term(expr.rhs.clone(), true)? {
			Value::Identifier(identifier) => identifier,

			_ => {
				create_error!(
					self,
					expr.rhs.position(),
					InterpretErrorKind::ExpressionNotAssignable(errors::ExpressionNotAssignable(
						None
					))
				)
			}
		};

		let index = match self.evaluate_term(expr.lhs.clone(), false)? {
			Value::Number(index) => index.get_owned() as i64,
			value => {
				create_error!(
					self,
					expr.position(),
					InterpretErrorKind::CannotIndexValue(errors::CannotIndexValue {
						kind: (ValueKind::List, expr.rhs.position()),
						expected_index_kind: ValueKind::Number,
						index_kind: (value.kind(), expr.lhs.position()),
						because_negative: false
					})
				)
			}
		};

		if index.is_negative() {
			create_error!(
				self,
				expr.position(),
				InterpretErrorKind::CannotIndexValue(errors::CannotIndexValue {
					kind: (ValueKind::List, expr.rhs.position()),
					expected_index_kind: ValueKind::Number,
					index_kind: (value.kind(), expr.lhs.position()),
					because_negative: true
				})
			);
		}

		let index: usize = index.try_into()?;
		let mut mutator = match self.context.get(&list_name)? {
			Value::List(list) => list.get_owned(),

			value => {
				create_error!(
					self,
					expr.position(),
					InterpretErrorKind::ExpressionNotAssignable(errors::ExpressionNotAssignable(
						Some(value.kind())
					))
				)
			}
		};

		let mut entries = mutator.take();

		if index >= entries.len() {
			entries.resize(index + 1, Value::Empty);
		}
		entries[index] = value;

		mutator.write(entries);

		Ok(Value::Empty)
	}

	// TODO: split class logic into `execute_expression_assign_class`
	fn execute_expr_assign_obj(
		&mut self,
		expr: ast::BinaryExpr<'static>,
		value: Value
	) -> Result<Value> {
		let object_name = match self.evaluate_term(expr.lhs.clone(), true)? {
			Value::Identifier(identifier) => identifier,

			_ => {
				create_error!(
					self,
					expr.position(),
					InterpretErrorKind::ExpressionNotAssignable(errors::ExpressionNotAssignable(
						None
					))
				)
			}
		};

		let entry_name = match self.evaluate_term(expr.rhs.clone(), true)? {
			Value::Identifier(value) => value,
			Value::String(value) => value.get_owned(),

			value => {
				create_error!(
					self,
					expr.position(),
					InterpretErrorKind::CannotIndexValue(errors::CannotIndexValue {
						kind: (ValueKind::Object, expr.lhs.position()),
						expected_index_kind: ValueKind::String,
						index_kind: (value.kind(), expr.rhs.position()),
						because_negative: false
					})
				)
			}
		};

		// Here we're handling objects and classes at the same time
		// because the difference between them is minimal
		let (object_kind, mut fields, parent_fields) = match self.context.get(&object_name)? {
			Value::Object(ref mut obj) => (ValueKind::Object, obj.get_owned(), None),
			Value::ClassInstance(inst) => {
				(
					ValueKind::ClassInstance,
					inst.fields.clone(),
					Some(inst.class.fields.clone())
				)
			}

			value => {
				create_error!(
					self,
					expr.position(),
					InterpretErrorKind::ExpressionNotAssignable(errors::ExpressionNotAssignable(
						Some(value.kind())
					))
				)
			}
		};

		// `HeapMutator::get_mut` fails, so this is a workaround
		let mut map = fields.take();

		// Checking if the field that is to be reassigned is a function inside a class instance
		// (which is illegal)
		if object_kind == ValueKind::ClassInstance {
			let parent_fields = parent_fields.unwrap();

			if !map.contains_key(&entry_name) && !parent_fields.contains_key(&entry_name) {
				create_error!(
					self,
					expr.lhs.position(),
					InterpretErrorKind::FieldDoesntExist(errors::FieldDoesntExist(
						entry_name,
						expr.rhs.position()
					))
				)
			}

			macro_rules! check_fields {
				($fields:expr) => {
					match ($fields).get(&entry_name) {
						Some(val) if val.kind() == ValueKind::Function => {
							create_error!(
								self,
								expr.position(),
								InterpretErrorKind::CannotReassignClassFunctions(
									errors::CannotReassignClassFunctions
								)
							)
						}

						_ => {}
					}
				};
			}

			check_fields!(map);
			check_fields!(*parent_fields);
		}

		map.insert(entry_name, value);
		fields.write(map);

		Ok(Value::Empty)
	}

	fn execute_class_def(&mut self, stmt: ast::ClassDef<'static>) -> Result<Value> {
		// This is a hack to keep fields of `self` and the actual class in sync
		let mut fields = unsafe { self.memory.alloc(HashMap::new()).promote() };

		self.context.deeper();
		self.context.insert(
			String::from(META_NO_SELF_OVERRIDE),
			Value::Boolean(true.into())
		);
		self.context.insert(
			String::from("self"),
			values::RClass {
				name: stmt.name.value_owned(),
				fields: fields.clone()
			}
			.into()
		);

		let mut temp_fields = HashMap::new();
		for field in stmt.fields {
			let name = field.name().value_owned();
			let value = match field.value() {
				Some(e) => self.evaluate_expr(e, false)?,
				None => Value::Empty
			};

			temp_fields.insert(name, value);
		}
		fields.write(temp_fields);

		let class = values::RClass {
			name: stmt.name.value_owned(),
			fields
		};

		self.context.shallower();
		self.context
			.insert(stmt.name.value_owned(), class.clone().into());

		Ok(class.into())
	}

	fn evaluate_expr(&mut self, expr: ast::Expr<'static>, stop_on_ident: bool) -> Result<Value> {
		use ast::Expr;

		match expr {
			Expr::Unary(e) => self.evaluate_unary_expr(e, stop_on_ident),
			Expr::Binary(e) => self.evaluate_binary_expr(e, stop_on_ident),
			Expr::Term(t) => self.evaluate_term(t, stop_on_ident)
		}
	}

	fn evaluate_unary_expr(
		&mut self,
		expr: ast::UnaryExpr<'static>,
		stop_on_ident: bool
	) -> Result<Value> {
		use ast::UnaryOpKind as Op;
		use Value as V;

		let operand = self.evaluate_term(expr.operand.clone(), stop_on_ident)?;

		match (expr.op.kind, operand) {
			(Op::Minus, V::Number(v)) => Ok(V::Number((-v.get_owned()).into())),

			(Op::Not, V::Boolean(v)) => Ok(V::Boolean((!v.get()).into())),
			(Op::Not, V::String(v)) => Ok(V::Boolean(v.get().is_empty().into())),
			(Op::Not, V::Number(v)) => Ok(V::Boolean((v.get_owned() == 0.0).into())),
			(Op::Not, V::List(v)) => Ok(V::Boolean(v.get().is_empty().into())),
			(Op::Not, V::Function(_)) => Ok(V::Boolean(false.into())),
			(Op::Not, V::IntrinsicFunction(..)) => Ok(V::Boolean(false.into())),
			(Op::Not, V::Empty) => Ok(V::Boolean(true.into())),

			(op, operand) => {
				create_error!(
					self,
					expr.position(),
					InterpretErrorKind::UnsupportedUnary(errors::UnsupportedUnary {
						operator: (op, expr.op.position()),
						operand: (operand.kind(), expr.operand.position())
					})
				)
			}
		}
	}

	fn evaluate_binary_expr(
		&mut self,
		expr: ast::BinaryExpr<'static>,
		stop_on_ident: bool
	) -> Result<Value> {
		use ast::BinaryOpKind as Op;
		use Value as V;

		let expr_pos = expr.position();
		let lhs_pos = expr.lhs.position();
		let rhs_pos = expr.rhs.position();

		let lhs = self.evaluate_term(expr.lhs.clone(), stop_on_ident)?;
		// if performing an object access and rhs is a valid identifier,
		// essentially force the `stop_on_ident` to `true`
		let rhs = match (expr.op.kind == Op::Dot, identifier_from_term(&expr.rhs)) {
			(true, Some(ident)) => Value::Identifier(ident),
			_ => self.evaluate_term(expr.rhs, stop_on_ident)?
		};

		macro_rules! primitive_object_access {
			($lhs:expr, $key:expr) => {{
				let map = $lhs.dispatch_map();
				let function = map.get(&$key);

				if function.is_none() {
					create_error!(
						self,
						expr.lhs.position(),
						InterpretErrorKind::FieldDoesntExist(errors::FieldDoesntExist(
							$key, rhs_pos
						))
					);
				}

				let mut function = function.unwrap().to_owned();

				function.this = Some(Box::new($lhs.into()));

				Value::IntrinsicFunction(function.into())
			}};
		}

		let evaluated_expr = match (expr.op.kind, lhs, rhs) {
			(Op::Plus, V::Number(lhs), V::Number(rhs)) => V::Number(lhs + rhs),
			(Op::Minus, V::Number(lhs), V::Number(rhs)) => V::Number(lhs - rhs),
			(Op::Asterisk, V::Number(lhs), V::Number(rhs)) => V::Number(lhs * rhs),
			(Op::Slash, V::Number(lhs), V::Number(rhs)) => V::Number(lhs / rhs),
			(Op::Sign, V::Number(lhs), V::Number(rhs)) => V::Number(lhs % rhs),
			(Op::Gt, V::Number(lhs), V::Number(rhs)) => V::Boolean((lhs > rhs).into()),
			(Op::Lt, V::Number(lhs), V::Number(rhs)) => V::Boolean((lhs < rhs).into()),
			(Op::Gte, V::Number(lhs), V::Number(rhs)) => V::Boolean((lhs >= rhs).into()),
			(Op::Lte, V::Number(lhs), V::Number(rhs)) => V::Boolean((lhs <= rhs).into()),

			(Op::Plus, V::String(lhs), rhs) => V::String(format!("{}{}", lhs.get(), rhs).into()),

			(Op::And, V::Boolean(lhs), V::Boolean(rhs)) => V::Boolean(lhs & rhs),
			(Op::Or, V::Boolean(lhs), V::Boolean(rhs)) => V::Boolean(lhs | rhs),

			(Op::EqEq, lhs, rhs) => V::Boolean((lhs == rhs).into()),
			(Op::Neq, lhs, rhs) => V::Boolean((lhs != rhs).into()),

			(Op::LeftBracket, V::Number(lhs), V::List(rhs)) => rhs[lhs].clone(),

			(Op::Dot, V::Object(lhs), rhs @ (V::Identifier(_) | V::String(_))) => {
				let rhs = rhs.extract_ident();
				let entries = &**(lhs.get());

				entries.get(rhs).cloned().unwrap_or(Value::Empty)
			}

			(Op::Dot, V::Boolean(lhs), rhs @ (V::Identifier(_) | V::String(_))) => {
				let rhs = rhs.extract_ident().to_owned();
				primitive_object_access!(lhs, rhs)
			}
			(Op::Dot, V::String(lhs), rhs @ (V::Identifier(_) | V::String(_))) => {
				let rhs = rhs.extract_ident().to_owned();
				primitive_object_access!(lhs, rhs)
			}
			(Op::Dot, V::Number(lhs), rhs @ (V::Identifier(_) | V::String(_))) => {
				let rhs = rhs.extract_ident().to_owned();
				primitive_object_access!(lhs, rhs)
			}
			(Op::Dot, V::List(lhs), rhs @ (V::Identifier(_) | V::String(_))) => {
				let rhs = rhs.extract_ident().to_owned();
				primitive_object_access!(lhs, rhs)
			}

			(Op::Dot, V::Class(lhs), rhs @ (V::Identifier(_) | V::String(_))) => {
				let rhs = rhs.extract_ident().to_owned();
				let field = (*lhs.fields).get(&rhs).cloned();

				field.ok_or(create_error!(
					self,
					expr_pos,
					InterpretErrorKind::FieldDoesntExist(errors::FieldDoesntExist(
						rhs,
						rhs_pos
					));
					no_bail
				))?
			}
			(Op::Dot, V::ClassInstance(lhs), rhs @ (V::Identifier(_) | V::String(_))) => {
				let rhs = rhs.extract_ident().to_owned();

				let instance_fields = &*lhs.fields;
				let class_fields = &*lhs.class.fields;

				if let Some(val) = instance_fields.get(&rhs).cloned() {
					return Ok(val);
				}

				let mut field = class_fields.get(&rhs).cloned().ok_or(create_error!(
					self,
					expr_pos,
					InterpretErrorKind::FieldDoesntExist(errors::FieldDoesntExist(
						rhs,
						rhs_pos
					));
					no_bail
				))?;

				if let Value::Function(func) = &mut field {
					let has_arguments = func.ast.args.is_some();

					if has_arguments {
						let arg = func.ast.args.as_ref().map(|p| p.first().unwrap());
						let arg_name = arg.unwrap().value_owned();

						if arg_name == META_SELF {
							// Insert `self` into scope
							func.context.insert(String::from("self"), lhs.into());

							// Remove `self` argument from the function
							func.ast.args.as_mut().unwrap().pop_first();
						}
					}
				}

				field
			}

			(_, lhs, rhs) => {
				create_error!(
					self,
					expr_pos,
					InterpretErrorKind::UnsupportedBinary(errors::UnsupportedBinary {
						lhs: (lhs.kind(), lhs_pos),
						operator: (expr.op.kind, expr.op.position()),
						rhs: (rhs.kind(), rhs_pos)
					})
				)
			}
		};

		Ok(evaluated_expr)
	}

	fn evaluate_term(&mut self, term: ast::Term<'static>, stop_on_ident: bool) -> Result<Value> {
		use ast::Term;

		let position = term.position();

		match term {
			Term::Expr(e) => self.evaluate_expr(*e, stop_on_ident),
			Term::ParenExpr(e) => self.evaluate_expr(*e.expr, stop_on_ident),

			Term::Lit(lit) => Ok(lit.into()),
			Term::Ident(ident) => {
				let ident = ident.value_owned();

				match stop_on_ident {
					true => Ok(Value::Identifier(ident)),
					false => {
						let error = create_error!(self, position, InterpretErrorKind::VariableDoesntExist(
							errors::VariableDoesntExist(ident.clone())
						); no_bail);

						self.context.get(&ident).map_err(|_| error)
					}
				}
			}
			Term::Func(func) => self.evaluate_func(func),
			Term::List(list) => self.evaluate_list(list),
			Term::Obj(obj) => self.evaluate_obj(obj),
			Term::Extern(ext) => self.evaluate_extern(ext),

			Term::Call(call) => self.evaluate_call(call),
			// TODO: that's a horrible way of doing this
			Term::IndexAcc(acc) => {
				eprintln!("This is a reminder to rewrite `Interpreter::evaluate_term` ASAP");

				let op = unsafe {
					let token = &acc._lb as &dyn prog_parser::Token;
					let bin_op = ast::BinaryOp::try_from(token).unwrap();

					std::mem::transmute::<ast::BinaryOp<'_>, ast::BinaryOp<'static>>(bin_op)
				};

				self.evaluate_expr(
					ast::Expr::Binary(ast::BinaryExpr {
						lhs: *acc.list,
						op,
						rhs: ast::Term::Expr(acc.index)
					}),
					false
				)
			}
			Term::FieldAcc(acc) => {
				eprintln!("This is a reminder to rewrite `Interpreter::evaluate_term` ASAP");

				let op = unsafe {
					let token = &acc._dot as &dyn prog_parser::Token;
					let bin_op = ast::BinaryOp::try_from(token).unwrap();

					std::mem::transmute::<ast::BinaryOp<'_>, ast::BinaryOp<'static>>(bin_op)
				};

				self.evaluate_expr(
					ast::Expr::Binary(ast::BinaryExpr {
						lhs: *acc.object,
						op,
						rhs: ast::Term::Ident(acc.field)
					}),
					false
				)
			}
		}
	}

	fn evaluate_func(&mut self, func: ast::Func<'static>) -> Result<Value> {
		let context = {
			let mut context = Context::new();

			context.deref_mut().flags = self.context.deref().flags;
			context.deref_mut().parent = Some(self.context.clone()); //* NOTE: This only clones the reference

			if self.no_self_override() {
				context.insert(
					String::from(META_NO_SELF_OVERRIDE),
					Value::Boolean(true.into())
				);

				context.insert(String::from(META_SELF), self.context.get(META_SELF)?);
			}

			context
		};

		let converted = RFunction {
			ast: Box::new(func),

			source: self.source.to_owned(),
			file: self.file.to_owned(),

			context
		};

		Ok(Value::Function(converted))
	}

	fn evaluate_extern(&mut self, ext: ast::Extern<'static>) -> Result<Value> {
		if !self.context.deref().flags.externs_allowed {
			create_error!(
				self,
				ext.position(),
				InterpretErrorKind::ContextDisallowed(errors::ContextDisallowed {
					thing: String::from("externs"),
					plural: true
				})
			)
		}

		let value_pos = ext.value.position();
		let value = match self.evaluate_expr(*ext.value, false)? {
			Value::String(v) => v.get_owned(),
			v => {
				create_error!(
					self,
					value_pos,
					InterpretErrorKind::InvalidExternArgument(errors::InvalidExternArgument(
						v.kind()
					))
				)
			}
		};

		self.externs.get(&value).cloned().ok_or(create_error!(
			self,
			value_pos,
			InterpretErrorKind::NonExistentExternItem(errors::NonExistentExternItem(
				value
			));
			no_bail
		))
	}

	fn evaluate_obj(&mut self, obj: ast::Obj<'static>) -> Result<Value> {
		use std::collections::HashMap;

		let fields = obj.fields.map(|p| p.unwrap_items()).unwrap_or_default();

		let mut value_map = HashMap::new();
		let mut position_map: HashMap<String, prog_parser::Position> = HashMap::new();

		for field in fields {
			let position = field.position();
			let name = field.name.value_owned();
			let value = self.evaluate_expr(field.value, false)?;

			if value_map.insert(name.clone(), value).is_some() {
				let definition_pos = position_map
					.get(&name)
					.unwrap_or_else(|| {
						unreachable!(
							"Position for entry `{}` does not exist in the position map",
							name
						)
					})
					.to_owned();

				create_error!(
					self,
					position,
					InterpretErrorKind::DuplicateObjectEntry(errors::DuplicateObjectEntry {
						entry_name: name,
						definition_pos
					})
				);
			}

			position_map.insert(name, position);
		}

		let allocated = unsafe { self.memory.alloc(value_map).promote() };
		Ok(Value::Object(allocated.into()))
	}

	fn evaluate_list(&mut self, list: ast::List<'static>) -> Result<Value> {
		let items = list.items.map(|p| p.unwrap_items()).unwrap_or_default();
		let mut values = vec![];

		for expr in items {
			let value = self.evaluate_expr(expr, false)?;
			values.push(value);
		}

		let allocated = unsafe { self.memory.alloc(values).promote() };
		Ok(Value::List(allocated.into()))
	}

	fn evaluate_call(&mut self, call: ast::Call<'static>) -> Result<Value> {
		let call_site = {
			let func = call.func.position();
			let _lp = call._lp.position();
			let args = call
				.args
				.clone()
				.map(|args| *args)
				.map(|args| args.map(|item| item.position(), |punct| punct.position()));
			let _rp = call._rp.position();

			CallSite {
				source: self.source.clone(),
				file: self.file.clone(),

				func,
				_lp,
				args,
				_rp
			}
		};

		let call_pos = call.position();
		let call_args_pos = call.args.as_ref().map(|p| p.position());
		let call_args = call
			.args
			.as_ref()
			.map(|p| p.items())
			.unwrap_or_default()
			.into_iter()
			.map(|arg| self.evaluate_expr(arg.clone(), false))
			.collect::<Result<Vec<_>>>()?;

		let original_expr = *call.func.clone();
		let function_pos = original_expr.position();

		let function_expr = self.evaluate_term(*call.func, false)?;

		let convert_error = |e: arg_parser::ArgumentParseError| {
			match e {
				arg_parser::ArgumentParseError::CountMismatch {
					expected,
					end_boundary,
					got
				} => {
					create_error!(
						self,
						call_args_pos.unwrap(),
						InterpretErrorKind::ArgumentCountMismatch(errors::ArgumentCountMismatch {
							expected,
							end_boundary,
							got,
							fn_call_pos: function_pos,
							fn_def_args_pos: None
						});
						no_bail
					)
				}

				arg_parser::ArgumentParseError::IncorrectType {
					index,
					expected,
					got
				} => {
					let arg = *call
						.args
						.as_ref()
						.unwrap()
						.items()
						.get(index)
						.unwrap_or_else(|| {
							panic!("Argument at index `{index}` does not exist when it should")
						});

					create_error!(
						self,
						arg.position(),
						InterpretErrorKind::ArgumentTypeMismatch(errors::ArgumentTypeMismatch {
							expected,
							got,
							function_pos
						});
						no_bail
					)
				}
			}
		};

		if let Value::IntrinsicFunction(function) = function_expr {
			let call_args = function
				.arguments
				.verify(&call_args)
				.map_err(convert_error)?;

			self.context.deeper();
			let result = function.call(self, call_args, call_site);
			self.context.shallower();

			return result;
		}

		if let Value::Function(mut function) = function_expr {
			let got_len = call.args.as_ref().map_or(0, |p| p.len());
			let expected_len = function.ast.args.as_ref().map_or(0, |p| p.len());

			if got_len != expected_len && expected_len == 0 {
				create_error!(
					self,
					call_args_pos.unwrap(),
					InterpretErrorKind::ArgumentCountMismatch(errors::ArgumentCountMismatch {
						expected: 0..0,
						end_boundary: true,
						got: got_len,
						fn_call_pos: function_pos,
						fn_def_args_pos: None
					})
				)
			}

			if got_len != expected_len {
				create_error!(
					self,
					call.args.unwrap().position(),
					InterpretErrorKind::ArgumentCountMismatch(errors::ArgumentCountMismatch {
						expected: expected_len..expected_len,
						end_boundary: true,
						got: got_len,
						fn_call_pos: call_pos,
						fn_def_args_pos: function.ast.args.map(|p| p.position())
					})
				);
			}

			let source = self.source.clone();
			let file = self.file.clone();

			// Function execution
			{
				use std::mem;

				self.context.deeper();
				self.source = function.source.clone();
				self.file = function.file.clone();

				mem::swap(&mut self.context, &mut function.context);

				// `self` variable insertion
				{
					// `no_override` indicates whether `self` has already been inferred
					// and does not need reassignment
					let no_override = self
						.context
						.get(META_NO_SELF_OVERRIDE)
						.unwrap_or(Value::Boolean(false.into()));

					match no_override {
						Value::Boolean(bool) if bool.get_owned() => {}

						_ => {
							self.context
								.insert(String::from("self"), Value::Function(function.clone()));
						}
					}
				}

				let arg_iter = function
					.ast
					.args
					.map(|p| p.unwrap_items())
					.unwrap_or_default()
					.into_iter()
					.map(|arg| arg.value_owned())
					.zip(call_args);
				for (name, value) in arg_iter {
					self.context.insert(name, value);
				}

				let exec_result = self.execute(
					ast::Program {
						stmts: function.ast.stmts
					},
					false
				);

				let result = exec_result.map_err(|err| {
					// downcast `anyhow` error into `InterpretError`
					let downcasted = err.downcast::<InterpretError>().unwrap_or_else(|_| {
						panic!("Function execution returned a non-`InterpretError` error")
					});

					// TODO: nest the error inside of `FunctionPanicked` error instead of printing it directly
					// TODO: fold the call stack of recursive functions
					// print it
					downcasted.eprint();

					// replace the original error with a new one
					anyhow::anyhow!(errors::InterpretError::new(
						source.clone(),
						file.clone(),
						call_site.whole(),
						InterpretErrorKind::FunctionPanicked(errors::FunctionPanicked)
					))
				});

				mem::swap(&mut self.context, &mut function.context);

				self.context.shallower();
				self.source = source;
				self.file = file;

				return result;
			}
		}

		// If calling a `Class`, treat it as constructing a class instance
		if let Value::Class(class) = function_expr {
			let props = {
				use arg_parser::{Arg, ArgList, ParsedArg};

				let parser = ArgList::new(vec![Arg::Required("props", ValueKind::Object)]);

				let arg = parser
					.verify(&call_args)
					.map_err(convert_error)?
					.remove("props")
					.unwrap();

				if let ParsedArg::Regular(arg) = arg {
					if let Value::Object(arg) = arg {
						arg.get().get_owned()
					} else {
						panic!("`props is not an object");
					}
				} else {
					panic!("`props` is a variadic argument");
				}
			};

			let filtered_class_fields = class
				.fields
				.iter()
				.filter(|&(_, value)| value.kind() == ValueKind::Empty)
				.map(|(name, value)| (name.to_owned(), value.to_owned()))
				.collect::<HashMap<_, _>>();
			let mut instance_fields = HashMap::new();

			for (prop_name, prop_value) in props {
				if !filtered_class_fields.contains_key(&prop_name) {
					create_error!(
						self,
						function_pos,
						InterpretErrorKind::FieldDoesntExist(errors::FieldDoesntExist(
							prop_name,
							call.args.unwrap().position()
						))
					)
				}

				instance_fields.insert(prop_name, prop_value);
			}

			if instance_fields.keys().len() < filtered_class_fields.len() {
				let missing_fields = class
					.fields
					.keys()
					.filter(|name| !instance_fields.contains_key(*name))
					.map(|name| name.to_owned())
					.collect::<Vec<_>>();

				create_error!(
					self,
					call.args.unwrap().position(),
					InterpretErrorKind::NonExhaustiveClassConstruction(
						errors::NonExhaustiveClassConstruction(missing_fields)
					)
				)
			}

			let allocated = unsafe { self.memory.alloc(instance_fields).promote() };

			return Ok(values::RClassInstance {
				class,
				fields: allocated
			}
			.into());
		}

		create_error!(
			self,
			original_expr.position(),
			InterpretErrorKind::ExpressionNotCallable(errors::ExpressionNotCallable(
				function_expr.kind()
			))
		)
	}

	/// Indicates whether the `self` variable should be overriden
	/// with the lowermost function in the context hierarchy
	fn no_self_override(&self) -> bool {
		let value = self.context.get(META_NO_SELF_OVERRIDE);

		if value.is_err() {
			return false;
		}

		let value = value.unwrap();

		match value {
			Value::Boolean(value) => value.get_owned(),

			value => {
				eprintln!(
					"INTERPRETER WARNING: `{}` was expected to be of type `{}`, but it is `{}`",
					META_NO_SELF_OVERRIDE,
					ValueKind::Boolean,
					value.kind()
				);

				false
			}
		}
	}
}

impl Default for Interpreter {
	fn default() -> Self { Self::new() }
}
