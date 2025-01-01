pub mod arg_parser;
pub mod context;
pub mod errors;
pub mod intrinsics;
pub mod values;

use std::collections::HashMap;
use anyhow::Result;
use ariadne::Span as _;

use context::Context;
pub use errors::{InterpretError, InterpretErrorKind};
use halloc::Memory;
use prog_parser::ast;
use values::{CallSite, RFunction};
pub use values::{RPrimitive, Value, ValueKind};

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

fn identifier_from_term(term: &ast::expressions::Term) -> Option<String> {
	match term {
		ast::expressions::Term::Identifier(value, _) => Some(value.to_owned()),
		ast::expressions::Term::Expression(value) => {
			if let ast::expressions::Expression::Term(ref value) = value.as_ref() {
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
		ast: ast::Program,
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

	pub fn execute(&mut self, ast: ast::Program, keep_marker: bool) -> Result<Value> {
		for statement in ast.statements {
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

	pub fn execute_statement(&mut self, statement: ast::Statement) -> Result<Value> {
		match statement {
			ast::Statement::VariableDefine(stmt) => self.execute_variable_define(stmt),
			ast::Statement::VariableAssign(stmt) => self.execute_variable_assign(stmt),
			ast::Statement::DoBlock(stmt) => self.execute_do_block(stmt),
			ast::Statement::Return(stmt) => self.execute_return(stmt),
			ast::Statement::Call(stmt) => self.evaluate_call(stmt),
			ast::Statement::WhileLoop(stmt) => self.execute_while_loop(stmt),
			ast::Statement::Break(stmt) => self.execute_break(stmt),
			ast::Statement::Continue(stmt) => self.execute_continue(stmt),
			ast::Statement::If(stmt) => self.execute_if(stmt),
			ast::Statement::ExpressionAssign(stmt) => self.execute_expression_assign(stmt),
			ast::Statement::ClassDefine(stmt) => self.execute_class_define(stmt)
		}
	}

	fn execute_variable_define(&mut self, statement: ast::VariableDefine) -> Result<Value> {
		let evaluated_value = match statement.value {
			None => Value::Empty,
			Some(expression) => self.evaluate_expression(expression, false)?
		};

		self.context.insert(statement.name.0, evaluated_value);
		Ok(Value::Empty)
	}

	fn execute_variable_assign(&mut self, statement: ast::VariableAssign) -> Result<Value> {
		let variable_name = statement.name.0;

		let evaluated_value = self.evaluate_expression(statement.value, false)?;
		let update_result = self.context.update(variable_name.clone(), evaluated_value);

		if update_result.is_err() {
			create_error!(
				self,
				statement.name.1,
				InterpretErrorKind::VariableDoesntExist(errors::VariableDoesntExist(variable_name))
			);
		}

		Ok(Value::Empty)
	}

	fn execute_do_block(&mut self, statement: ast::DoBlock) -> Result<Value> {
		self.context.deeper();
		let result = self.execute(
			ast::Program {
				statements: statement.statements
			},
			false
		);
		self.context.shallower();

		result
	}

	fn execute_return(&mut self, statement: ast::Return) -> Result<Value> {
		let value = match statement.expression {
			Some(expression) => self.evaluate_expression(expression, false)?,
			None => Value::Empty
		};

		Ok(Value::ControlFlow(values::ControlFlow::Return(Box::new(
			value
		))))
	}

	fn execute_while_loop(&mut self, statement: ast::WhileLoop) -> Result<Value> {
		let mut evaluated = self.evaluate_expression(statement.condition.clone(), false)?;

		while evaluated.is_truthy() {
			self.context.deeper();
			let result = self.execute(
				ast::Program {
					statements: statement.statements.clone()
				},
				true
			)?;
			self.context.shallower();

			if let Value::ControlFlow(ref ctrl) = result {
				match ctrl {
					values::ControlFlow::Return(_) => return Ok(result),
					values::ControlFlow::Break => break,
					values::ControlFlow::Continue => {
						evaluated = self.evaluate_expression(statement.condition.clone(), false)?;
						continue;
					}
				};
			}

			evaluated = self.evaluate_expression(statement.condition.clone(), false)?;
		}

		Ok(Value::Empty)
	}

	fn execute_break(&mut self, _statement: ast::Break) -> Result<Value> {
		Ok(Value::ControlFlow(values::ControlFlow::Break))
	}

	fn execute_continue(&mut self, _statement: ast::Continue) -> Result<Value> {
		Ok(Value::ControlFlow(values::ControlFlow::Continue))
	}

	fn execute_if(&mut self, statement: ast::If) -> Result<Value> {
		let evaluated = self.evaluate_expression(statement.condition, false)?;

		if evaluated.is_truthy() {
			self.context.deeper();
			let result = self.execute(
				ast::Program {
					statements: statement.statements
				},
				true
			)?;
			self.context.shallower();

			if result.kind() == ValueKind::ControlFlow {
				return Ok(result);
			}

			return Ok(Value::Empty);
		}

		for branch in statement.elseif_branches {
			let evaluated = self.evaluate_expression(branch.condition, false)?;

			if evaluated.is_truthy() {
				self.context.deeper();
				let result = self.execute(
					ast::Program {
						statements: branch.statements
					},
					true
				)?;
				self.context.shallower();

				if result.kind() == ValueKind::ControlFlow {
					return Ok(result);
				}

				return Ok(Value::Empty);
			}
		}

		if let Some(branch) = statement.else_branch {
			self.context.deeper();
			let result = self.execute(
				ast::Program {
					statements: branch.statements
				},
				true
			)?;
			self.context.shallower();

			if result.kind() == ValueKind::ControlFlow {
				return Ok(result);
			}

			return Ok(Value::Empty);
		}

		Ok(Value::Empty)
	}

	fn execute_expression_assign(&mut self, statement: ast::ExpressionAssign) -> Result<Value> {
		use ast::expressions::operators::BinaryOperator as Op;

		let expression = match statement.expression {
			ast::Expression::Binary(expression) => expression,
			_ => {
				create_error!(
					self,
					statement.expression.position(),
					InterpretErrorKind::ExpressionNotAssignable(errors::ExpressionNotAssignable(
						None
					))
				)
			}
		};

		if !matches!(expression.operator.0, Op::ListAccess | Op::ObjectAccess) {
			create_error!(
				self,
				expression.position,
				InterpretErrorKind::ExpressionNotAssignable(errors::ExpressionNotAssignable(None))
			);
		}

		let value = self.evaluate_expression(statement.value, false)?;

		if expression.operator.0 == Op::ListAccess {
			self.execute_expression_assign_list(expression, value)
		} else {
			self.execute_expression_assign_object(expression, value)
		}
	}

	fn execute_expression_assign_list(
		&mut self,
		expression: ast::expressions::Binary,
		value: Value
	) -> Result<Value> {
		let ast::expressions::Binary {
			lhs,
			rhs,
			operator: _,
			position
		} = expression;

		let list_name = match self.evaluate_term(rhs.clone(), true)? {
			Value::Identifier(identifier) => identifier,
			_ => {
				create_error!(
					self,
					position,
					InterpretErrorKind::ExpressionNotAssignable(errors::ExpressionNotAssignable(
						None
					))
				)
			}
		};

		let index = match self.evaluate_term(lhs.clone(), false)? {
			Value::Number(index) => index.get_owned() as i64,
			value => {
				create_error!(
					self,
					position,
					InterpretErrorKind::CannotIndexValue(errors::CannotIndexValue {
						kind: (ValueKind::List, rhs.position()),
						expected_index_kind: ValueKind::Number,
						index_kind: (value.kind(), lhs.position()),
						because_negative: false
					})
				)
			}
		};

		if index.is_negative() {
			create_error!(
				self,
				position,
				InterpretErrorKind::CannotIndexValue(errors::CannotIndexValue {
					kind: (ValueKind::List, rhs.position()),
					expected_index_kind: ValueKind::Number,
					index_kind: (value.kind(), lhs.position()),
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
					position,
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
	fn execute_expression_assign_object(
		&mut self,
		expression: ast::expressions::Binary,
		value: Value
	) -> Result<Value> {
		let ast::expressions::Binary {
			lhs,
			rhs,
			operator: _,
			position
		} = expression.clone();

		let object_name = match self.evaluate_term(lhs.clone(), true)? {
			Value::Identifier(identifier) => identifier,
			_ => {
				create_error!(
					self,
					position,
					InterpretErrorKind::ExpressionNotAssignable(errors::ExpressionNotAssignable(
						None
					))
				)
			}
		};

		let entry_name = match self.evaluate_term(rhs.clone(), true)? {
			Value::Identifier(value) => value,
			Value::String(value) => value.get_owned(),

			value => {
				create_error!(
					self,
					position,
					InterpretErrorKind::CannotIndexValue(errors::CannotIndexValue {
						kind: (ValueKind::Object, lhs.position()),
						expected_index_kind: ValueKind::String,
						index_kind: (value.kind(), rhs.position()),
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
					position,
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
					lhs.position(),
					InterpretErrorKind::FieldDoesntExist(errors::FieldDoesntExist(
						entry_name,
						rhs.position()
					))
				)
			}

			macro_rules! check_fields {
				($fields:expr) => {
					match ($fields).get(&entry_name) {
						Some(val) if val.kind() == ValueKind::Function => {
							create_error!(
								self,
								position,
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

	fn execute_class_define(&mut self, statement: ast::ClassDefine) -> Result<Value> {
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
				name: statement.name.clone(),
				fields: fields.clone()
			}
			.into()
		);

		let mut temp_fields = HashMap::new();
		for field in statement.fields {
			let value = field
				.value
				.map_or(Ok(Value::Empty), |val| self.evaluate_expression(val, false))?;

			temp_fields.insert(field.name.0, value);
		}
		fields.write(temp_fields);

		let class = values::RClass {
			name: statement.name.clone(),
			fields
		};

		self.context.shallower();
		self.context.insert(statement.name, class.clone().into());

		Ok(class.into())
	}

	fn evaluate_expression(
		&mut self,
		expression: ast::Expression,
		stop_on_ident: bool
	) -> Result<Value> {
		use ast::expressions::*;

		match expression {
			Expression::Unary(expression) => {
				self.evaluate_unary_expression(
					expression.operator,
					expression.operand,
					stop_on_ident
				)
			}
			Expression::Binary(expression) => {
				self.evaluate_binary_expression(
					expression.lhs,
					expression.operator,
					expression.rhs,
					stop_on_ident
				)
			}
			Expression::Term(term) => self.evaluate_term(term, stop_on_ident),
			Expression::Empty(_) => Ok(Value::Empty)
		}
	}

	fn evaluate_unary_expression(
		&mut self,
		operator: (ast::expressions::operators::UnaryOperator, ast::Position),
		operand: ast::expressions::Term,
		stop_on_ident: bool
	) -> Result<Value> {
		use ast::expressions::operators::UnaryOperator as Op;
		use Value as V;

		let operator_pos = operator.1;
		let operand_pos = operand.position();
		let whole_pos = ast::Position::new(operator_pos.start(), operand_pos.end());

		let evaluated_operand = self.evaluate_term(operand.clone(), stop_on_ident)?;

		match (operator.0, evaluated_operand) {
			(Op::Minus, V::Number(v)) => Ok(V::Number((-v.get_owned()).into())),

			(Op::Not, V::Boolean(v)) => Ok(V::Boolean((!v.get()).into())),
			(Op::Not, V::String(v)) => Ok(V::Boolean(v.get().is_empty().into())),
			(Op::Not, V::Number(v)) => Ok(V::Boolean((v.get_owned() == 0.0).into())),
			(Op::Not, V::List(v)) => Ok(V::Boolean(v.get().is_empty().into())),
			(Op::Not, V::Function(_)) => Ok(V::Boolean(false.into())),
			(Op::Not, V::IntrinsicFunction(..)) => Ok(V::Boolean(false.into())),
			(Op::Not, V::Empty) => Ok(V::Boolean(true.into())),

			(_, evaluated_operand) => {
				create_error!(
					self,
					whole_pos,
					InterpretErrorKind::UnsupportedUnary(errors::UnsupportedUnary {
						operator,
						operand: (evaluated_operand.kind(), operand.position())
					})
				)
			}
		}
	}

	fn evaluate_binary_expression(
		&mut self,
		lhs: ast::expressions::Term,
		operator: (ast::expressions::operators::BinaryOperator, ast::Position),
		rhs: ast::expressions::Term,
		stop_on_ident: bool
	) -> Result<Value> {
		use ast::expressions::operators::BinaryOperator as Op;
		use Value as V;

		let lhs_position = lhs.position();
		let rhs_position = rhs.position();

		let whole_position = ast::Position::new(lhs_position.start(), rhs_position.end());

		let evaluated_lhs = self.evaluate_term(lhs.clone(), stop_on_ident)?;
		// if performing an object access and rhs is a valid identifier,
		// essentially force the `stop_on_ident` to `true`
		let evaluated_rhs = match (operator.0 == Op::ObjectAccess, identifier_from_term(&rhs)) {
			(true, Some(ident)) => Value::Identifier(ident),
			_ => self.evaluate_term(rhs, stop_on_ident)?
		};

		macro_rules! primitive_object_access {
			($lhs:expr, $key:expr) => {{
				let map = $lhs.dispatch_map();
				let function = map.get(&$key);

				if function.is_none() {
					create_error!(
						self,
						lhs_position,
						InterpretErrorKind::FieldDoesntExist(errors::FieldDoesntExist(
							$key,
							rhs_position
						))
					);
				}

				let mut function = function.unwrap().to_owned();

				function.this = Some(Box::new($lhs.into()));

				Value::IntrinsicFunction(function.into())
			}};
		}

		let evaluated_expr = match (operator.0, evaluated_lhs, evaluated_rhs) {
			(Op::Add, V::Number(lhs), V::Number(rhs)) => V::Number(lhs + rhs),
			(Op::Subtract, V::Number(lhs), V::Number(rhs)) => V::Number(lhs - rhs),
			(Op::Divide, V::Number(lhs), V::Number(rhs)) => V::Number(lhs / rhs),
			(Op::Multiply, V::Number(lhs), V::Number(rhs)) => V::Number(lhs * rhs),
			(Op::Modulo, V::Number(lhs), V::Number(rhs)) => V::Number(lhs % rhs),
			(Op::Gt, V::Number(lhs), V::Number(rhs)) => V::Boolean((lhs > rhs).into()),
			(Op::Lt, V::Number(lhs), V::Number(rhs)) => V::Boolean((lhs < rhs).into()),
			(Op::Gte, V::Number(lhs), V::Number(rhs)) => V::Boolean((lhs >= rhs).into()),
			(Op::Lte, V::Number(lhs), V::Number(rhs)) => V::Boolean((lhs <= rhs).into()),

			(Op::Add, V::String(lhs), rhs) => V::String(format!("{}{}", lhs.get(), rhs).into()),

			(Op::And, V::Boolean(lhs), V::Boolean(rhs)) => V::Boolean(lhs & rhs),
			(Op::Or, V::Boolean(lhs), V::Boolean(rhs)) => V::Boolean(lhs | rhs),

			(Op::EqEq, lhs, rhs) => V::Boolean((lhs == rhs).into()),
			(Op::NotEq, lhs, rhs) => V::Boolean((lhs != rhs).into()),

			(Op::ListAccess, V::Number(lhs), V::List(rhs)) => rhs[lhs].clone(),

			(Op::ObjectAccess, V::Object(lhs), rhs @ (V::Identifier(_) | V::String(_))) => {
				let rhs = rhs.extract_identifier();
				let entries = &**(lhs.get());

				entries.get(rhs).cloned().unwrap_or(Value::Empty)
			}

			(Op::ObjectAccess, V::Boolean(lhs), rhs @ (V::Identifier(_) | V::String(_))) => {
				let rhs = rhs.extract_identifier().to_owned();
				primitive_object_access!(lhs, rhs)
			}
			(Op::ObjectAccess, V::String(lhs), rhs @ (V::Identifier(_) | V::String(_))) => {
				let rhs = rhs.extract_identifier().to_owned();
				primitive_object_access!(lhs, rhs)
			}
			(Op::ObjectAccess, V::Number(lhs), rhs @ (V::Identifier(_) | V::String(_))) => {
				let rhs = rhs.extract_identifier().to_owned();
				primitive_object_access!(lhs, rhs)
			}
			(Op::ObjectAccess, V::List(lhs), rhs @ (V::Identifier(_) | V::String(_))) => {
				let rhs = rhs.extract_identifier().to_owned();
				primitive_object_access!(lhs, rhs)
			}

			(Op::ObjectAccess, V::Class(lhs), rhs @ (V::Identifier(_) | V::String(_))) => {
				let rhs = rhs.extract_identifier().to_owned();
				let field = (*lhs.fields).get(&rhs).cloned();

				field.ok_or(create_error!(
					self,
					whole_position,
					InterpretErrorKind::FieldDoesntExist(errors::FieldDoesntExist(
						rhs,
						rhs_position
					));
					no_bail
				))?
			}
			(Op::ObjectAccess, V::ClassInstance(lhs), rhs @ (V::Identifier(_) | V::String(_))) => {
				let rhs = rhs.extract_identifier().to_owned();

				let instance_fields = &*lhs.fields;
				let class_fields = &*lhs.class.fields;

				if let Some(val) = instance_fields.get(&rhs).cloned() {
					return Ok(val);
				}

				let mut field = class_fields.get(&rhs).cloned().ok_or(create_error!(
					self,
					whole_position,
					InterpretErrorKind::FieldDoesntExist(errors::FieldDoesntExist(
						rhs,
						rhs_position
					));
					no_bail
				))?;

				if let Value::Function(func) = &mut field {
					let has_arguments = !func.ast.arguments.is_empty();

					if has_arguments {
						let first_argument_name = &func.ast.arguments.first().unwrap().0;

						if first_argument_name == META_SELF {
							// Insert `self` into scope
							func.context.insert(String::from("self"), lhs.into());

							// Remove `self` argument from the function
							func.ast.arguments.remove(0);
						}
					}
				}

				field
			}

			(_, evaluated_lhs, evaluated_rhs) => {
				create_error!(
					self,
					whole_position,
					InterpretErrorKind::UnsupportedBinary(errors::UnsupportedBinary {
						lhs: (evaluated_lhs.kind(), lhs_position),
						operator,
						rhs: (evaluated_rhs.kind(), rhs_position)
					})
				)
			}
		};

		Ok(evaluated_expr)
	}

	fn evaluate_term(
		&mut self,
		term: ast::expressions::Term,
		stop_on_ident: bool
	) -> Result<Value> {
		use ast::expressions::*;

		let position = term.position();

		match term {
			Term::Extern(ext) => self.evaluate_extern(ext),
			Term::Object(obj) => self.evaluate_object(obj),
			Term::List(list) => self.evaluate_list(list),
			Term::Call(call) => self.evaluate_call(call),
			Term::Function(func) => self.evaluate_function(func),
			Term::Literal(lit) => Ok(lit.into()),
			Term::Identifier(ident, _) => {
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
			Term::Expression(value) => self.evaluate_expression(*value, stop_on_ident)
		}
	}

	fn evaluate_function(&mut self, function: ast::expressions::Function) -> Result<Value> {
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
			ast: Box::new(function),

			source: self.source.to_owned(),
			file: self.file.to_owned(),

			context
		};

		Ok(Value::Function(converted))
	}

	fn evaluate_extern(&mut self, ext: ast::expressions::Extern) -> Result<Value> {
		if !self.context.deref().flags.externs_allowed {
			create_error!(
				self,
				ext.1,
				InterpretErrorKind::ContextDisallowed(errors::ContextDisallowed {
					thing: String::from("externs"),
					plural: true
				})
			)
		}

		let value_pos = ext.0.position();
		let value = match self.evaluate_expression(*ext.0, false)? {
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

	fn evaluate_object(&mut self, object: ast::expressions::Object) -> Result<Value> {
		use std::collections::HashMap;

		let mut value_map = HashMap::new();
		let mut position_map: HashMap<String, ast::Position> = HashMap::new();

		for entry in object.0 {
			let name = entry.name;
			let value = self.evaluate_expression(entry.value, false)?;

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
					entry.position,
					InterpretErrorKind::DuplicateObjectEntry(errors::DuplicateObjectEntry {
						entry_name: name,
						definition_pos
					})
				);
			}

			position_map.insert(name, entry.position);
		}

		let allocated = unsafe { self.memory.alloc(value_map).promote() };
		Ok(Value::Object(allocated.into()))
	}

	fn evaluate_list(&mut self, list: ast::expressions::List) -> Result<Value> {
		let mut values = vec![];

		for expression in list.0 {
			let value = self.evaluate_expression(expression, false)?;
			values.push(value);
		}

		let allocated = unsafe { self.memory.alloc(values).promote() };
		Ok(Value::List(allocated.into()))
	}

	fn evaluate_call(&mut self, call: ast::expressions::Call) -> Result<Value> {
		let call_site = CallSite {
			source: self.source.clone(),
			file: self.file.clone(),

			args_pos: call.arguments.1,
			func_pos: call.function.position(),
			whole_pos: call.position
		};

		let call_args_pos = call_site.args_pos;
		let call_args = call
			.arguments
			.0
			.clone()
			.into_iter()
			.map(|arg| self.evaluate_expression(arg, false))
			.collect::<Result<Vec<_>>>()?;

		let original_expr = *call.function.clone();
		let function_pos = original_expr.position();

		let function_expr = self.evaluate_expression(*call.function, false)?;

		let convert_error = |e: arg_parser::ArgumentParseError| {
			match e {
				arg_parser::ArgumentParseError::CountMismatch {
					expected,
					end_boundary,
					got
				} => {
					create_error!(
						self,
						call_args_pos,
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
					let argument = call.arguments.0.get(index).unwrap_or_else(|| {
						panic!("Argument at index `{index}` does not exist when it should")
					});

					create_error!(
						self,
						argument.position(),
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
			let got_len = call_args.len();
			let expected_len = function.ast.arguments.len();

			if got_len != expected_len && expected_len == 0 {
				create_error!(
					self,
					call_args_pos,
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
				let first_arg = function.ast.arguments.first().unwrap();
				let last_arg = function.ast.arguments.last().unwrap();

				let fn_call_pos = function_pos;
				let fn_def_args_pos = Some(ast::Position::new(first_arg.1.start(), last_arg.1.end()));

				create_error!(
					self,
					call_args_pos,
					InterpretErrorKind::ArgumentCountMismatch(errors::ArgumentCountMismatch {
						expected: expected_len..expected_len,
						end_boundary: true,
						got: got_len,
						fn_call_pos,
						fn_def_args_pos
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

				let argument_iter = function.ast.arguments.into_iter().zip(call_args);
				for ((arg_name, _), arg_value) in argument_iter {
					self.context.insert(arg_name, arg_value);
				}

				let exec_result = self.execute(
					ast::Program {
						statements: function.ast.statements
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
						call_site.whole_pos,
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
							call_args_pos
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
					call_args_pos,
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
