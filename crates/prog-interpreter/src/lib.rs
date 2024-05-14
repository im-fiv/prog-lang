pub mod arg_parser;
pub mod context;
pub mod errors;
pub mod intrinsics;
pub mod values;

pub use values::{RuntimePrimitive, RuntimeValue, RuntimeValueKind};
pub use errors::{InterpretError, InterpretErrorKind};

use context::RuntimeContext;
use values::{RuntimeFunction, CallSite};

use prog_parser::ast;
use anyhow::{Result, bail};

/// Only to be used inside the interpreter impl
macro_rules! create_error {
	($self:expr, $position:expr, $kind:expr) => {
		bail!(errors::InterpretError::new(
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
		},

		_ => None
	}
}

fn is_value_truthy(rv: &RuntimeValue) -> bool {
	use RuntimeValue as Rv;
	
	match rv {
		// Values that are inexpensive to clone can be cloned
		Rv::Boolean(value) => value.borrow().owned(),
		Rv::String(value) => !value.borrow().value().is_empty(),
		Rv::Number(value) => value.borrow().owned() != 0.0,
		Rv::List(value) => !value.borrow().value().is_empty(),
		Rv::Object(value) => !value.borrow().value().is_empty(),

		Rv::Function(_) => true,
		Rv::IntrinsicFunction(..) => true,

		Rv::Empty => false,

		Rv::Identifier(..) => unreachable!("RuntimeValue of kind Identifier"),
		Rv::Marker(..) => unreachable!("RuntimeValue of kind Marker")
	}
}

#[derive(Debug)]
pub struct Interpreter {
	pub context: RuntimeContext,
	source: String,
	file: String
}

impl Interpreter {
	pub fn new(source: String, file: String) -> Self {
		Self {
			context: RuntimeContext::new(),
			source,
			file
		}
	}

	pub fn execute(&mut self, ast: ast::Program, keep_marker: bool) -> Result<RuntimeValue> {
		for statement in ast.statements {
			let result = self.execute_statement(statement)?;

			// In case of `return`, `break`, and `continue` statements
			if let RuntimeValue::Marker(ref marker) = result {
				if keep_marker {
					return Ok(result);
				}

				if let values::MarkerKind::Return(value) = marker {
					return Ok(*value.to_owned());
				}

				return Ok(RuntimeValue::Empty);
			}
		}

		Ok(RuntimeValue::Empty)
	}

	pub fn execute_statement(&mut self, statement: ast::Statement) -> Result<RuntimeValue> {
		match statement {
			ast::Statement::VariableDefine(statement) => self.execute_variable_define(statement),
			ast::Statement::VariableAssign(statement) => self.execute_variable_assign(statement),
			ast::Statement::DoBlock(statement) => self.execute_do_block(statement),
			ast::Statement::Return(statement) => self.execute_return(statement),
			ast::Statement::Call(statement) => self.evaluate_call(statement),
			ast::Statement::WhileLoop(statement) => self.execute_while_loop(statement),
			ast::Statement::Break(statement) => self.execute_break(statement),
			ast::Statement::Continue(statement) => self.execute_continue(statement),
			ast::Statement::If(statement) => self.execute_if(statement),
			ast::Statement::ExpressionAssign(statement) => self.execute_expression_assign(statement)
		}
	}

	fn execute_variable_define(&mut self, statement: ast::VariableDefine) -> Result<RuntimeValue> {
		let evaluated_value = match statement.value {
			None => RuntimeValue::Empty,
			Some(expression) => self.evaluate_expression(expression, false)?
		};

		if self.context.insert_value(statement.name.0.clone(), evaluated_value).is_err() {
			create_error!(self, statement.name.1, InterpretErrorKind::ValueAlreadyExists(
				errors::ValueAlreadyExists(statement.name.0)
			));
		}

		Ok(RuntimeValue::Empty)
	}

	fn execute_variable_assign(&mut self, statement: ast::VariableAssign) -> Result<RuntimeValue> {
		let evaluated_value = self.evaluate_expression(statement.value, false)?;

		if self.context.update_value(statement.name.0.clone(), evaluated_value).is_err() {
			create_error!(self, statement.name.1, InterpretErrorKind::ValueDoesntExist(
				errors::ValueDoesntExist(statement.name.0)
			));
		}

		Ok(RuntimeValue::Empty)
	}

	fn execute_do_block(&mut self, statement: ast::DoBlock) -> Result<RuntimeValue> {
		self.context.deeper();
		let result = self.execute(ast::Program {
			statements: statement.statements
		}, false);
		self.context.shallower();

		result
	}

	fn execute_return(&mut self, statement: ast::Return) -> Result<RuntimeValue> {
		let value = match statement.expression {
			Some(expression) => self.evaluate_expression(expression, false)?,
			None => RuntimeValue::Empty
		};

		Ok(RuntimeValue::Marker(
			values::MarkerKind::Return(Box::new(value))
		))
	}

	fn execute_while_loop(&mut self, statement: ast::WhileLoop) -> Result<RuntimeValue> {
		let mut evaluated = self.evaluate_expression(statement.condition.clone(), false)?;

		while is_value_truthy(&evaluated) {
			self.context.deeper();
			let result = self.execute(ast::Program {
				statements: statement.statements.clone()
			}, true)?;
			self.context.shallower();

			if let RuntimeValue::Marker(ref marker) = result {
				match marker {
					values::MarkerKind::Return(_) => return Ok(result),
					values::MarkerKind::Break => break,
					values::MarkerKind::Continue => {
						evaluated = self.evaluate_expression(statement.condition.clone(), false)?;
						continue;
					}
				};
			}

			evaluated = self.evaluate_expression(statement.condition.clone(), false)?;
		}

		Ok(RuntimeValue::Empty)
	}

	fn execute_break(&mut self, _statement: ast::Break) -> Result<RuntimeValue> {
		Ok(RuntimeValue::Marker(values::MarkerKind::Break))
	}

	fn execute_continue(&mut self, _statement: ast::Continue) -> Result<RuntimeValue> {
		Ok(RuntimeValue::Marker(values::MarkerKind::Continue))
	}

	fn execute_if(&mut self, statement: ast::If) -> Result<RuntimeValue> {
		let evaluated = self.evaluate_expression(statement.condition, false)?;

		if is_value_truthy(&evaluated) {
			self.context.deeper();
			let result = self.execute(ast::Program {
				statements: statement.statements
			}, true)?;
			self.context.shallower();

			if result.kind() == RuntimeValueKind::Marker {
				return Ok(result);
			}

			return Ok(RuntimeValue::Empty);
		}

		for branch in statement.elseif_branches {
			let evaluated = self.evaluate_expression(branch.condition, false)?; 

			if is_value_truthy(&evaluated) {
				self.context.deeper();
				let result = self.execute(ast::Program {
					statements: branch.statements
				}, true)?;
				self.context.shallower();

				if result.kind() == RuntimeValueKind::Marker {
					return Ok(result);
				}

				return Ok(RuntimeValue::Empty);
			}
		}

		if let Some(branch) = statement.else_branch {
			self.context.deeper();
			let result = self.execute(ast::Program {
				statements: branch.statements
			}, true)?;
			self.context.shallower();

			if result.kind() == RuntimeValueKind::Marker {
				return Ok(result);
			}

			return Ok(RuntimeValue::Empty);
		}

		Ok(RuntimeValue::Empty)
	}

	fn execute_expression_assign(&mut self, statement: ast::ExpressionAssign) -> Result<RuntimeValue> {
		use ast::expressions::operators::BinaryOperator as Op;

		let expression = match statement.expression {
			ast::Expression::Binary(expression) => expression,
			_ => create_error!(
				self,
				statement.expression.position(),
				InterpretErrorKind::ExpressionNotAssignable(
					errors::ExpressionNotAssignable(None)
				)
			)
		};

		if !matches!(expression.operator.0, Op::ListAccess | Op::ObjectAccess) {
			create_error!(
				self,
				expression.position,
				InterpretErrorKind::ExpressionNotAssignable(
					errors::ExpressionNotAssignable(None)
				)
			);
		}

		let value = self.evaluate_expression(statement.value, false)?;

		if expression.operator.0 == Op::ListAccess {
			self.execute_expression_assign_list(expression, value)
		} else {
			self.execute_expression_assign_object(expression, value)
		}
	}

	fn execute_expression_assign_list(&mut self, expression: ast::expressions::Binary, value: RuntimeValue) -> Result<RuntimeValue> {
		let ast::expressions::Binary { lhs, rhs, operator: _, position } = expression;

		let list_name = match self.evaluate_term(rhs.clone(), true)? {
			RuntimeValue::Identifier(identifier) => identifier,
			_ => create_error!(self, position, InterpretErrorKind::ExpressionNotAssignable(
				errors::ExpressionNotAssignable(None)
			))
		};

		let index = match self.evaluate_term(lhs.clone(), false)? {
			RuntimeValue::Number(index) => index.borrow().owned() as i64,
			value => create_error!(self, position, InterpretErrorKind::CannotIndexValue(
				errors::CannotIndexValue {
					kind: (RuntimeValueKind::List, rhs.position()),
					expected_index_kind: RuntimeValueKind::Number,
					index_kind: (value.kind(), lhs.position()),
					because_negative: false
				}
			))
		};

		if index.is_negative() {
			create_error!(self, position, InterpretErrorKind::CannotIndexValue(
				errors::CannotIndexValue {
					kind: (RuntimeValueKind::List, rhs.position()),
					expected_index_kind: RuntimeValueKind::Number,
					index_kind: (value.kind(), lhs.position()),
					because_negative: true
				}
			));
		}

		let index: usize = index.try_into()?;
		let inner_list = match self.context.get_value_mut(&list_name)? {
			RuntimeValue::List(inner_list) => inner_list,
			value => create_error!(self, position, InterpretErrorKind::ExpressionNotAssignable(
				errors::ExpressionNotAssignable(Some(value.kind()))
			))
		};

		let mut inner_list_borrowed = inner_list.borrow_mut();

		if index >= inner_list_borrowed.0.len() {
			inner_list_borrowed.0.resize(index + 1, RuntimeValue::Empty);
		}

		inner_list_borrowed.0[index] = value;

		Ok(RuntimeValue::Empty)
	}

	fn execute_expression_assign_object(&mut self, expression: ast::expressions::Binary, value: RuntimeValue) -> Result<RuntimeValue> {
		let ast::expressions::Binary { lhs, rhs, operator: _, position } = expression.clone();

		let object_name = match self.evaluate_term(lhs.clone(), true)? {
			RuntimeValue::Identifier(identifier) => identifier,
			_ => create_error!(self, position, InterpretErrorKind::ExpressionNotAssignable(
				errors::ExpressionNotAssignable(None)
			))
		};

		let entry_name = match self.evaluate_term(rhs.clone(), true)? {
			RuntimeValue::Identifier(value) => value,
			RuntimeValue::String(value) => value.borrow().owned(),

			value => create_error!(self, position, InterpretErrorKind::CannotIndexValue(
				errors::CannotIndexValue {
					kind: (RuntimeValueKind::Object, lhs.position()),
					expected_index_kind: RuntimeValueKind::String,
					index_kind: (value.kind(), rhs.position()),
					because_negative: false
				}
			))
		};

		let inner_object = match self.context.get_value_mut(&object_name)? {
			RuntimeValue::Object(inner_object) => inner_object,
			
			value => create_error!(self, position, InterpretErrorKind::ExpressionNotAssignable(
				errors::ExpressionNotAssignable(Some(value.kind()))
			))
		};

		let mut inner_object_borrowed = inner_object.borrow_mut();
		inner_object_borrowed.0.insert(entry_name, value);

		Ok(RuntimeValue::Empty)
	}

	fn evaluate_expression(&mut self, expression: ast::Expression, stop_on_ident: bool) -> Result<RuntimeValue> {
		use ast::expressions::*;
		
		match expression {
			Expression::Unary(expression) => self.evaluate_unary_expression(expression.operator, expression.operand, stop_on_ident),
			Expression::Binary(expression) => self.evaluate_binary_expression(expression.lhs, expression.operator, expression.rhs, stop_on_ident),
			Expression::Term(term) => self.evaluate_term(term, stop_on_ident),
			Expression::Empty(_) => Ok(RuntimeValue::Empty)
		}
	}

	fn evaluate_unary_expression(
		&mut self,
		operator: (ast::expressions::operators::UnaryOperator, ast::Position),
		operand: ast::expressions::Term,
		stop_on_ident: bool
	) -> Result<RuntimeValue> {
		use ast::expressions::operators::UnaryOperator as Op;
		use RuntimeValue as Rv;

		let operator_pos = operator.1.clone();
		let operand_pos = operand.position();
		let whole_pos = operator_pos.start..operand_pos.end;

		let evaluated_operand = self.evaluate_term(operand.clone(), stop_on_ident)?;

		match (operator.0, evaluated_operand) {
			(Op::Minus, Rv::Number(value)) => Ok(Rv::Number(values::RuntimeNumber::from(-value.borrow().owned()).into())),

			// TODO: yucky code, should find a way to clean this up
			(Op::Not, Rv::Boolean(value)) => Ok(Rv::Boolean(values::RuntimeBoolean::from(!value.borrow().owned()).into())),
			(Op::Not, Rv::String(value)) => Ok(Rv::Boolean(values::RuntimeBoolean::from(value.borrow().owned().is_empty()).into())),
			(Op::Not, Rv::Number(value)) => Ok(Rv::Boolean(values::RuntimeBoolean::from(value.borrow().owned() == 0.0).into())),
			(Op::Not, Rv::List(value)) => Ok(Rv::Boolean(values::RuntimeBoolean::from(value.borrow().owned().is_empty()).into())),
			(Op::Not, Rv::Function(_)) => Ok(Rv::Boolean(values::RuntimeBoolean::from(false).into())),
			(Op::Not, Rv::IntrinsicFunction(..)) => Ok(Rv::Boolean(values::RuntimeBoolean::from(false).into())),
			(Op::Not, Rv::Empty) => Ok(Rv::Boolean(values::RuntimeBoolean::from(true).into())),

			(_, evaluated_operand) => create_error!(self, whole_pos, InterpretErrorKind::UnsupportedUnary(
				errors::UnsupportedUnary {
					operator,
					operand: (evaluated_operand.kind(), operand.position())
				}
			))
		}
	}

	fn evaluate_binary_expression(
		&mut self,
		lhs: ast::expressions::Term,
		operator: (ast::expressions::operators::BinaryOperator, ast::Position),
		rhs: ast::expressions::Term,
		stop_on_ident: bool
	) -> Result<RuntimeValue> {
		use ast::expressions::operators::BinaryOperator as Op;
		use RuntimeValue as Rv;

		let lhs_position = lhs.position();
		let rhs_position = rhs.position();

		let whole_position = lhs_position.start..rhs_position.end;

		let evaluated_lhs = self.evaluate_term(lhs.clone(), stop_on_ident)?;
		// if performing an object access and rhs is a valid identifier,
		// essentially force the `stop_on_ident` to `true`
		let evaluated_rhs = match (operator.0 == Op::ObjectAccess, identifier_from_term(&rhs)) {
			(true, Some(ident)) => RuntimeValue::Identifier(ident),
			_ => self.evaluate_term(rhs, stop_on_ident)?
		};

		macro_rules! primitive_object_access {
			($lhs:expr, $key:expr) => {{
				let map = $lhs.borrow().dispatch_map();
				let function = map.get(&$key);

				if function.is_none() {
					create_error!(self, lhs_position, InterpretErrorKind::FieldDoesntExist(
						errors::FieldDoesntExist($key, rhs_position)
					));
				}

				let mut function = function
					.unwrap()
					.to_owned();

				function.this = Some(Box::new(
					$lhs.into()
				));

				Ok(RuntimeValue::IntrinsicFunction(
					function.into()
				))
			}};
		}

		match (operator.0, evaluated_lhs, evaluated_rhs) {
			(Op::Plus, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Number(values::RuntimeNumber::from(lhs.borrow().owned() + rhs.borrow().owned()).into())),
			(Op::Minus, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Number(values::RuntimeNumber::from(lhs.borrow().owned() - rhs.borrow().owned()).into())),
			(Op::Divide, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Number(values::RuntimeNumber::from(lhs.borrow().owned() / rhs.borrow().owned()).into())),
			(Op::Multiply, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Number(values::RuntimeNumber::from(lhs.borrow().owned() * rhs.borrow().owned()).into())),
			(Op::Modulo, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Number(values::RuntimeNumber::from(lhs.borrow().owned() % rhs.borrow().owned()).into())),
			(Op::Gt, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Boolean(values::RuntimeBoolean::from(lhs.borrow().owned() > rhs.borrow().owned()).into())),
			(Op::Lt, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Boolean(values::RuntimeBoolean::from(lhs.borrow().owned() < rhs.borrow().owned()).into())),
			(Op::Gte, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Boolean(values::RuntimeBoolean::from(lhs.borrow().owned() >= rhs.borrow().owned()).into())),
			(Op::Lte, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Boolean(values::RuntimeBoolean::from(lhs.borrow().owned() <= rhs.borrow().owned()).into())),

			(Op::Plus, Rv::String(lhs), rhs) => Ok(Rv::String(values::RuntimeString::from(format!("{}{}", lhs.borrow(), rhs)).into())),

			(Op::And, Rv::Boolean(lhs), Rv::Boolean(rhs)) => Ok(Rv::Boolean(values::RuntimeBoolean::from(lhs.borrow().owned() && rhs.borrow().owned()).into())),
			(Op::Or, Rv::Boolean(lhs), Rv::Boolean(rhs)) => Ok(Rv::Boolean(values::RuntimeBoolean::from(lhs.borrow().owned() || rhs.borrow().owned()).into())),

			(Op::EqEq, Rv::Boolean(lhs), Rv::Boolean(rhs)) => Ok(Rv::Boolean(values::RuntimeBoolean::from(lhs == rhs).into())),
			(Op::EqEq, Rv::String(lhs), Rv::String(rhs)) => Ok(Rv::Boolean(values::RuntimeBoolean::from(lhs == rhs).into())),
			(Op::EqEq, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Boolean(values::RuntimeBoolean::from(lhs == rhs).into())),
			(Op::EqEq, Rv::List(lhs), Rv::List(rhs)) => Ok(Rv::Boolean(values::RuntimeBoolean::from(lhs == rhs).into())),
			(Op::EqEq, Rv::Function(lhs), Rv::Function(rhs)) => Ok(Rv::Boolean(values::RuntimeBoolean::from(lhs == rhs).into())),
			(Op::EqEq, Rv::IntrinsicFunction(lhs), Rv::IntrinsicFunction(rhs)) => Ok(Rv::Boolean(values::RuntimeBoolean::from(lhs == rhs).into())),
			(Op::EqEq, Rv::Empty, Rv::Empty) => Ok(Rv::Boolean(values::RuntimeBoolean::from(true).into())),

			(Op::NotEq, Rv::Boolean(lhs), Rv::Boolean(rhs)) => Ok(Rv::Boolean(values::RuntimeBoolean::from(lhs != rhs).into())),
			(Op::NotEq, Rv::String(lhs), Rv::String(rhs)) => Ok(Rv::Boolean(values::RuntimeBoolean::from(lhs != rhs).into())),
			(Op::NotEq, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Boolean(values::RuntimeBoolean::from(lhs != rhs).into())),
			(Op::NotEq, Rv::List(lhs), Rv::List(rhs)) => Ok(Rv::Boolean(values::RuntimeBoolean::from(lhs != rhs).into())),
			(Op::NotEq, Rv::Function(lhs), Rv::Function(rhs)) => Ok(Rv::Boolean(values::RuntimeBoolean::from(lhs != rhs).into())),
			(Op::NotEq, Rv::IntrinsicFunction(lhs), Rv::IntrinsicFunction(rhs)) => Ok(Rv::Boolean(values::RuntimeBoolean::from(lhs != rhs).into())),
			(Op::NotEq, Rv::Empty, Rv::Empty) => Ok(Rv::Boolean(values::RuntimeBoolean::from(false).into())),

			(Op::ListAccess, Rv::Number(lhs), Rv::List(rhs)) => Ok(
				rhs
					.borrow()
					.value()
					.get(lhs.borrow().owned() as usize)
					.unwrap_or(&RuntimeValue::Empty)
					.to_owned()
			),

			// TODO: implement behavior for when an object has a user-defined function with the same name
			(Op::ObjectAccess, Rv::Object(lhs), Rv::Identifier(rhs)) => Ok(
				lhs
					.borrow()
					.value()
					.get(&rhs)
					.unwrap_or(&RuntimeValue::Empty)
					.to_owned()
			),
			(Op::ObjectAccess, Rv::Object(lhs), Rv::String(rhs)) => Ok(
				lhs
					.borrow()
					.value()
					.get(&rhs.borrow().owned())
					.unwrap_or(&RuntimeValue::Empty)
					.to_owned()
			),

			(Op::ObjectAccess, Rv::Boolean(lhs), Rv::Identifier(rhs)) => primitive_object_access!(lhs, rhs),
			(Op::ObjectAccess, Rv::String(lhs), Rv::Identifier(rhs)) => primitive_object_access!(lhs, rhs),
			(Op::ObjectAccess, Rv::Number(lhs), Rv::Identifier(rhs)) => primitive_object_access!(lhs, rhs),
			(Op::ObjectAccess, Rv::List(lhs), Rv::Identifier(rhs)) => primitive_object_access!(lhs, rhs),

			(Op::ObjectAccess, Rv::Boolean(lhs), Rv::String(rhs)) => primitive_object_access!(lhs, rhs.borrow().owned()),
			(Op::ObjectAccess, Rv::String(lhs), Rv::String(rhs)) => primitive_object_access!(lhs, rhs.borrow().owned()),
			(Op::ObjectAccess, Rv::Number(lhs), Rv::String(rhs)) => primitive_object_access!(lhs, rhs.borrow().owned()),
			(Op::ObjectAccess, Rv::List(lhs), Rv::String(rhs)) => primitive_object_access!(lhs, rhs.borrow().owned()),

			(Op::EqEq, _, _) => Ok(Rv::Boolean(values::RuntimeBoolean::from(false).into())),
			(Op::NotEq, _, _) => Ok(Rv::Boolean(values::RuntimeBoolean::from(true).into())),

			(_, evaluated_lhs, evaluated_rhs) => create_error!(self, whole_position, InterpretErrorKind::UnsupportedBinary(
				errors::UnsupportedBinary {
					lhs: (evaluated_lhs.kind(), lhs_position),
					operator,
					rhs: (evaluated_rhs.kind(), rhs_position)
				}
			))
		}
	}

	fn evaluate_term(&mut self, term: ast::expressions::Term, stop_on_ident: bool) -> Result<RuntimeValue> {
		use ast::expressions::*;

		let position = term.position();

		match term {
			Term::Object(value) => self.evaluate_object(value),
			Term::List(value) => self.evaluate_list(value),
			Term::Call(value) => self.evaluate_call(value),
			Term::Function(value) => self.evaluate_function(value),
			Term::Literal(value) => Ok(value.into()),
			Term::Identifier(value, _) => match stop_on_ident {
				true => Ok(RuntimeValue::Identifier(value)),
				false => self.context
					.get_value(&value)
					.map_err(|_|
						create_error!(self, position, InterpretErrorKind::ValueDoesntExist(
							errors::ValueDoesntExist(value)
						); no_bail)
					)
			},
			Term::Expression(value) => self.evaluate_expression(*value, stop_on_ident)
		}
	}

	fn evaluate_function(&self, function: ast::expressions::Function) -> Result<RuntimeValue> {
		let converted = RuntimeFunction {
			ast: Box::new(function),
			source: self.source.to_owned(),
			file: self.file.to_owned()
		};

		Ok(RuntimeValue::Function(
			converted.into()
		))
	}

	fn evaluate_object(&mut self, object: ast::expressions::Object) -> Result<RuntimeValue> {
		use std::collections::HashMap;

		let mut value_map = HashMap::new();
		let mut position_map: HashMap<String, std::ops::Range<usize>> = HashMap::new();

		for entry in object.0 {
			let name = entry.name;
			let value = self.evaluate_expression(entry.value, false)?;
			
			if value_map.insert(name.clone(), value).is_some() {
				let definition_pos = position_map
					.get(&name)
					.unwrap_or_else(|| unreachable!("Position for entry `{}` does not exist in the position map", name))
					.to_owned();

				create_error!(self, entry.position, InterpretErrorKind::DuplicateObjectEntry(
					errors::DuplicateObjectEntry {
						entry_name: name,
						definition_pos
					}
				));
			}

			position_map.insert(name, entry.position);
		}

		Ok(RuntimeValue::Object(
			values::RuntimeObject::from(value_map).into()
		))
	}

	fn evaluate_list(&mut self, list: ast::expressions::List) -> Result<RuntimeValue> {
		let mut values = vec![];

		for expression in list.0 {
			let value = self.evaluate_expression(expression, false)?;
			values.push(value);
		}

		Ok(RuntimeValue::List(
			values::RuntimeList::from(values).into()
		))
	}

	fn evaluate_call(&mut self, call: ast::expressions::Call) -> Result<RuntimeValue> {
		use std::iter::zip;

		let call_site = CallSite {
			source: self.source.clone(),
			file: self.file.clone(),
			position: call.position
		};

		let call_arguments_pos = call.arguments.1;
		let call_arguments = call.arguments.0
			.clone()
			.into_iter()
			.map(|arg| self.evaluate_expression(arg, false))
			.collect::<Result<Vec<_>>>()?;

		let original_expression = *call.function.clone();
		let function_pos = original_expression.position();

		let function_expression = self.evaluate_expression(*call.function, false)?;

		if let RuntimeValue::IntrinsicFunction(function) = function_expression {
			let function = function.borrow();

			let convert_error = |e: arg_parser::ArgumentParseError| {
				match e {
					arg_parser::ArgumentParseError::CountMismatch { expected, end_boundary, got } => create_error!(
						self,
						call_arguments_pos,
						InterpretErrorKind::ArgumentCountMismatch(errors::ArgumentCountMismatch {
							expected,
							end_boundary,
							got,
							fn_call_pos: function_pos,
							fn_def_args_pos: None
						});
						no_bail
					),

					arg_parser::ArgumentParseError::IncorrectType { index, expected, got } => {
						let argument = call
							.arguments
							.0
							.get(index)
							.unwrap_or_else(|| unreachable!("Argument at index `{index}` does not exist when it should"));

						create_error!(
							self,
							argument.position(),
							InterpretErrorKind::ArgumentTypeMismatch(errors::ArgumentTypeMismatch {
								expected, got, function_pos
							});
							no_bail
						)
					}
				}
			};

			let call_arguments = function
				.arguments
				.verify(&call_arguments)
				.map_err(convert_error)?;
			
			self.context.deeper();
			let result = function.call(&mut self.context, call_arguments, call_site);
			self.context.shallower();

			return result;
		}

		if let RuntimeValue::Function(function) = function_expression {
			let function = function
				.borrow()
				.to_owned();

			let got_len = call_arguments.len();
			let expected_len = function.ast.arguments.len();
			
			if got_len != expected_len {
				let first_arg = function
					.ast
					.arguments
					.first()
					.unwrap();

				let last_arg = function
					.ast
					.arguments
					.last()
					.unwrap();

				let fn_call_pos = function_pos;
				let fn_def_args_pos = Some(
					(first_arg.1.start)..(last_arg.1.end)
				);

				create_error!(self, call_arguments_pos, InterpretErrorKind::ArgumentCountMismatch(
					errors::ArgumentCountMismatch {
						expected: expected_len..expected_len,
						end_boundary: true,
						got: got_len,
						fn_call_pos,
						fn_def_args_pos
					}
				));
			}

			let source = self.source.clone();
			let file = self.file.clone();
			
			// Function execution
			{
				self.context.deeper();
				self.source = function.source;
				self.file = function.file;

				for ((arg_name, _), arg_value) in zip(function.ast.arguments, call_arguments) {
					self.context.insert_value(arg_name.clone(), arg_value)?;
				}

				let result = self.execute(ast::Program {
					statements: function.ast.statements
				}, false);

				self.context.shallower();
				self.source = source;
				self.file = file;

				result
			}
		} else {
			create_error!(
				self,
				original_expression.position(),
				InterpretErrorKind::ExpressionNotCallable(
					errors::ExpressionNotCallable(function_expression.kind())
				)
			)
		}
	}
}