pub mod arg_parser;
pub mod context;
pub mod errors;
pub mod intrinsics;
pub mod values;

pub use values::{RuntimeValue, RuntimeValueKind};
pub use errors::{InterpretError, InterpretErrorKind};

use context::RuntimeContext;
use values::{RuntimeFunction, Identifier, CallSite};

use prog_parser::ast;
use anyhow::{Result, bail};

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
		Rv::Boolean(value) => *value,
		Rv::String(value) => !value.is_empty(),
		Rv::Number(value) => value != &0.0,
		Rv::List(value) => !value.is_empty(),
		Rv::Object(value) => !value.is_empty(),

		Rv::Function(_) => true,
		Rv::IntrinsicFunction(..) => true,

		Rv::Identifier(..) => unreachable!("RuntimeValue of kind Identifier"),
		Rv::Empty => false
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

	fn create_error(&self, position: ast::Position, kind: InterpretErrorKind) -> Result<RuntimeValue> {
		bail!(errors::InterpretError::new(
			self.source.clone(),
			self.file.clone(),
			position,
			kind
		))
	}

	pub fn execute(&mut self, ast: ast::Program) -> Result<RuntimeValue> {
		for statement in ast.statements {
			let result = self.execute_statement(statement)?;

			if !matches!(result, RuntimeValue::Empty) {
				return Ok(result);
			}
		}

		Ok(RuntimeValue::Empty)
	}

	pub fn execute_statement(&mut self, statement: ast::Statement) -> Result<RuntimeValue> {
		match statement {
			ast::Statement::VariableDefine { name, value, position } => self.execute_variable_define(name, value, position),
			ast::Statement::VariableAssign { name, value, position } => self.execute_variable_assign(name, value, position),
			ast::Statement::DoBlock(statements, position) => self.execute_do_block(statements, position),

			ast::Statement::Return(expression, _position) => match expression {
				Some(expression) => self.evaluate_expression(expression, false),
				None => Ok(RuntimeValue::Empty)
			},

			ast::Statement::Call(call) => self.evaluate_call(call),
			ast::Statement::WhileLoop { condition, statements, position } => self.execute_while_loop(condition, statements, position),

			ast::Statement::Break(position) => self.create_error(
				position.clone(),
				InterpretErrorKind::UnsupportedStatement(
					errors::UnsupportedStatement(String::from("break"))
				)
			),

			ast::Statement::Continue(position) => self.create_error(
				position.clone(),
				InterpretErrorKind::UnsupportedStatement(
					errors::UnsupportedStatement(String::from("continue"))
				)
			),

			ast::Statement::If { condition, statements, elseif_branches, else_branch, position } => self.execute_if(condition, statements, elseif_branches, else_branch, position),
		
			ast::Statement::ExpressionAssign { expression, value, position} => self.execute_expression_assign(expression, value, position)
		}
	}

	fn execute_variable_define(&mut self, name: (String, ast::Position), value: Option<ast::Expression>, _position: ast::Position) -> Result<RuntimeValue> {
		let evaluated_value = match value {
			None => RuntimeValue::Empty,
			Some(expression) => self.evaluate_expression(expression, false)?
		};

		if self.context.insert_value(name.0.clone(), evaluated_value).is_err() {
			self.create_error(name.1.clone(), InterpretErrorKind::ValueAlreadyExists(
				errors::ValueAlreadyExists(name.0)
			))?;
		}

		Ok(RuntimeValue::Empty)
	}

	fn execute_variable_assign(&mut self, name: (String, ast::Position), value: ast::Expression, _position: ast::Position) -> Result<RuntimeValue> {
		let evaluated_value = self.evaluate_expression(value, false)?;

		if self.context.update_value(name.0.clone(), evaluated_value).is_err() {
			return self.create_error(name.1.clone(), InterpretErrorKind::ValueDoesntExist(
				errors::ValueDoesntExist(name.0)
			));
		}

		Ok(RuntimeValue::Empty)
	}

	fn execute_do_block(&mut self, statements: Vec<ast::Statement>, _position: ast::Position) -> Result<RuntimeValue> {
		self.context.deeper();
		let result = self.execute(ast::Program { statements });
		self.context.shallower();

		result
	}

	fn execute_while_loop(&mut self, condition: ast::Expression, statements: Vec<ast::Statement>, _position: ast::Position) -> Result<RuntimeValue> {
		let mut evaluated = self.evaluate_expression(condition.clone(), false)?;

		while is_value_truthy(&evaluated) {
			self.context.deeper();
			self.execute(ast::Program { statements: statements.clone() })?;
			self.context.shallower();

			evaluated = self.evaluate_expression(condition.clone(), false)?;
		}

		Ok(RuntimeValue::Empty)
	}

	fn execute_if(
		&mut self,
		condition: ast::Expression,
		statements: Vec<ast::Statement>,
		elseif_branches: Vec<ast::ConditionBranch>,
		else_branch: Option<ast::ConditionBranch>,
		_position: ast::Position
	) -> Result<RuntimeValue> {
		let evaluated = self.evaluate_expression(condition, false)?;

		if is_value_truthy(&evaluated) {
			self.context.deeper();
			self.execute(ast::Program { statements })?;
			self.context.shallower();

			return Ok(RuntimeValue::Empty);
		}

		for branch in elseif_branches {
			let evaluated = self.evaluate_expression(branch.condition, false)?; 

			if is_value_truthy(&evaluated) {
				self.context.deeper();
				self.execute(ast::Program { statements: branch.statements })?;
				self.context.shallower();

				return Ok(RuntimeValue::Empty);
			}
		}

		if let Some(branch) = else_branch {
			let evaluated = self.evaluate_expression(branch.condition, false)?; 

			if is_value_truthy(&evaluated) {
				self.context.deeper();
				self.execute(ast::Program { statements: branch.statements })?;
				self.context.shallower();

				return Ok(RuntimeValue::Empty);
			}
		}

		Ok(RuntimeValue::Empty)
	}

	fn execute_expression_assign(&mut self, expression: ast::Expression, value: ast::Expression, _position: ast::Position) -> Result<RuntimeValue> {
		use ast::expressions::operators::BinaryOperator as Op;

		let expression = match expression {
			ast::Expression::Binary(expression) => expression,
			_ => return self.create_error(
				expression.position(),
				InterpretErrorKind::ExpressionNotAssignable(
					errors::ExpressionNotAssignable(None)
				)
			)
		};

		if !matches!(expression.operator.0, Op::ListAccess | Op::ObjectAccess) {
			return self.create_error(
				expression.position,
				InterpretErrorKind::ExpressionNotAssignable(
					errors::ExpressionNotAssignable(None)
				)
			)
		}

		let value = self.evaluate_expression(value, false)?;

		if expression.operator.0 == Op::ListAccess {
			self.execute_expression_assign_list(expression, value)
		} else {
			self.execute_expression_assign_object(expression, value)
		}
	}

	fn execute_expression_assign_list(&mut self, expression: ast::expressions::Binary, value: RuntimeValue) -> Result<RuntimeValue> {
		let ast::expressions::Binary { lhs, rhs, operator: _, position } = expression;

		let list_name = match self.evaluate_term(rhs.clone(), true)? {
			RuntimeValue::Identifier(identifier) => identifier.0,
			_ => return self.create_error(position, InterpretErrorKind::ExpressionNotAssignable(
				errors::ExpressionNotAssignable(None)
			))
		};

		let mut inner_list = match self.context.get_value(&list_name)? {
			RuntimeValue::List(inner_list) => inner_list,
			value => return self.create_error(position, InterpretErrorKind::ExpressionNotAssignable(
				errors::ExpressionNotAssignable(Some(value.kind()))
			))
		};

		let index = match self.evaluate_term(lhs.clone(), false)? {
			RuntimeValue::Number(index) => index as i64,
			value => return self.create_error(position, InterpretErrorKind::CannotIndexValue(
				errors::CannotIndexValue {
					kind: (RuntimeValueKind::List, rhs.position()),
					expected_index_kind: RuntimeValueKind::Number,
					index_kind: (value.kind(), lhs.position()),
					because_negative: false
				}
			))
		};

		if index.is_negative() {
			return self.create_error(position, InterpretErrorKind::CannotIndexValue(
				errors::CannotIndexValue {
					kind: (RuntimeValueKind::List, rhs.position()),
					expected_index_kind: RuntimeValueKind::Number,
					index_kind: (value.kind(), lhs.position()),
					because_negative: true
				}
			))
		}

		let index: usize = index.try_into()?;

		if index >= inner_list.len() {
			inner_list.resize(index + 50, RuntimeValue::Empty);
		}

		inner_list[index] = value;

		if self.context.update_value(list_name.clone(), RuntimeValue::List(inner_list)).is_err() {
			return self.create_error(rhs.position(), InterpretErrorKind::ValueDoesntExist(
				errors::ValueDoesntExist(list_name)
			));
		}

		Ok(RuntimeValue::Empty)
	}

	fn execute_expression_assign_object(&mut self, expression: ast::expressions::Binary, value: RuntimeValue) -> Result<RuntimeValue> {
		let ast::expressions::Binary { lhs, rhs, operator: _, position } = expression.clone();

		let object_name = match self.evaluate_term(lhs.clone(), true)? {
			RuntimeValue::Identifier(identifier) => identifier.0,
			_ => return self.create_error(position, InterpretErrorKind::ExpressionNotAssignable(
				errors::ExpressionNotAssignable(None)
			))
		};

		let mut inner_object = match self.context.get_value(&object_name)? {
			RuntimeValue::Object(inner_object) => inner_object,
			value => return self.create_error(position, InterpretErrorKind::ExpressionNotAssignable(
				errors::ExpressionNotAssignable(Some(value.kind()))
			))
		};

		let entry_name = match self.evaluate_term(rhs.clone(), true)? {
			RuntimeValue::Identifier(value) => value.0,
			RuntimeValue::String(value) => value,

			value => return self.create_error(position, InterpretErrorKind::CannotIndexValue(
				errors::CannotIndexValue {
					kind: (RuntimeValueKind::Object, lhs.position()),
					expected_index_kind: RuntimeValueKind::String,
					index_kind: (value.kind(), rhs.position()),
					because_negative: false
				}
			))
		};

		inner_object.insert(entry_name, value);

		if self.context.update_value(object_name.clone(), RuntimeValue::Object(inner_object)).is_err() {
			return self.create_error(lhs.position(), InterpretErrorKind::ValueDoesntExist(
				errors::ValueDoesntExist(object_name)
			));
		}

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

		let evaluated_operand = self.evaluate_term(operand.clone(), stop_on_ident)?;

		match (operator.0, evaluated_operand) {
			(Op::Minus, Rv::Number(value)) => Ok(Rv::Number(-value)),

			(Op::Not, Rv::Boolean(value)) => Ok(Rv::Boolean(!value)),
			(Op::Not, Rv::String(value)) => Ok(Rv::Boolean(value.is_empty())),
			(Op::Not, Rv::Number(value)) => Ok(Rv::Boolean(value == 0.0)),
			(Op::Not, Rv::List(value)) => Ok(Rv::Boolean(value.is_empty())),
			(Op::Not, Rv::Function(_)) => Ok(Rv::Boolean(false)),
			(Op::Not, Rv::IntrinsicFunction(..)) => Ok(Rv::Boolean(false)),
			(Op::Not, Rv::Empty) => Ok(Rv::Boolean(true)),

			// `self.create_error` first argument isn't used in `UnsupportedUnary`
			(_, evaluated_operand) => return self.create_error(0..0, InterpretErrorKind::UnsupportedUnary(
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
		
		let mut evaluated_lhs = self.evaluate_term(lhs.clone(), stop_on_ident)?;
		let evaluated_lhs_forced = self.evaluate_term(lhs.clone(), false)?;

		let is_object_access = matches!(
			(&evaluated_lhs_forced, operator.0),
			(RuntimeValue::Object(_), Op::ObjectAccess)
		);

		let evaluated_rhs = match (is_object_access, identifier_from_term(&rhs)) {
			(true, Some(value)) => RuntimeValue::Identifier(Identifier(value)),
			_ => self.evaluate_term(rhs.clone(), false)?
		};

		if is_object_access {
			evaluated_lhs = evaluated_lhs_forced;
		}

		match (operator.0, evaluated_lhs, evaluated_rhs) {
			(Op::Plus, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Number(lhs + rhs)),
			(Op::Minus, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Number(lhs - rhs)),
			(Op::Divide, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Number(lhs / rhs)),
			(Op::Multiply, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Number(lhs * rhs)),
			(Op::Modulo, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Number(lhs % rhs)),
			(Op::Gt, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Boolean(lhs > rhs)),
			(Op::Lt, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Boolean(lhs < rhs)),
			(Op::Gte, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Boolean(lhs >= rhs)),
			(Op::Lte, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Boolean(lhs <= rhs)),

			(Op::Plus, Rv::String(lhs), rhs) => Ok(Rv::String(format!("{lhs}{rhs}"))),

			(Op::And, Rv::Boolean(lhs), Rv::Boolean(rhs)) => Ok(Rv::Boolean(lhs && rhs)),
			(Op::Or, Rv::Boolean(lhs), Rv::Boolean(rhs)) => Ok(Rv::Boolean(lhs || rhs)),

			(Op::EqEq, Rv::Boolean(lhs), Rv::Boolean(rhs)) => Ok(Rv::Boolean(lhs == rhs)),
			(Op::EqEq, Rv::String(lhs), Rv::String(rhs)) => Ok(Rv::Boolean(lhs == rhs)),
			(Op::EqEq, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Boolean(lhs == rhs)),
			(Op::EqEq, Rv::List(lhs), Rv::List(rhs)) => Ok(Rv::Boolean(lhs == rhs)),
			(Op::EqEq, Rv::Function(lhs), Rv::Function(rhs)) => Ok(Rv::Boolean(lhs == rhs)),
			(Op::EqEq, Rv::IntrinsicFunction(lhs), Rv::IntrinsicFunction(rhs)) => Ok(Rv::Boolean(lhs == rhs)),
			(Op::EqEq, Rv::Empty, Rv::Empty) => Ok(Rv::Boolean(true)),

			(Op::NotEq, Rv::Boolean(lhs), Rv::Boolean(rhs)) => Ok(Rv::Boolean(lhs != rhs)),
			(Op::NotEq, Rv::String(lhs), Rv::String(rhs)) => Ok(Rv::Boolean(lhs != rhs)),
			(Op::NotEq, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Boolean(lhs != rhs)),
			(Op::NotEq, Rv::List(lhs), Rv::List(rhs)) => Ok(Rv::Boolean(lhs != rhs)),
			(Op::NotEq, Rv::Function(lhs), Rv::Function(rhs)) => Ok(Rv::Boolean(lhs != rhs)),
			(Op::NotEq, Rv::IntrinsicFunction(lhs), Rv::IntrinsicFunction(rhs)) => Ok(Rv::Boolean(lhs == rhs)),
			(Op::NotEq, Rv::Empty, Rv::Empty) => Ok(Rv::Boolean(false)),

			(Op::ListAccess, Rv::Number(lhs), Rv::List(rhs)) => Ok(
				rhs
					.get(lhs as usize)
					.unwrap_or(&RuntimeValue::Empty)
					.to_owned()
			),

			(Op::ObjectAccess, Rv::Object(lhs), Rv::Identifier(rhs)) => Ok(
				lhs
					.get(&rhs.0)
					.unwrap_or(&RuntimeValue::Empty)
					.to_owned()
			),
			(Op::ObjectAccess, Rv::Object(lhs), Rv::String(rhs)) => Ok(
				lhs
					.get(&rhs)
					.unwrap_or(&RuntimeValue::Empty)
					.to_owned()
			),

			(Op::EqEq, _, _) => Ok(Rv::Boolean(false)),
			(Op::NotEq, _, _) => Ok(Rv::Boolean(true)),

			// (operator, lhs, rhs) => bail!("Cannot perform an unsupported binary operation `{} {} {}`", lhs.kind(), operator, rhs.kind())
			// `self.create_error` first argument isn't used in `UnsupportedBinary`
			(_, evaluated_lhs, evaluated_rhs) => return self.create_error(0..0, InterpretErrorKind::UnsupportedBinary(
				errors::UnsupportedBinary {
					lhs: (evaluated_lhs.kind(), lhs.position()),
					operator,
					rhs: (evaluated_rhs.kind(), rhs.position())
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
				true => Ok(RuntimeValue::Identifier(value.into())),
				false => self.context
					.get_value(&value)
					.map_err(|_|
						self.create_error(position, InterpretErrorKind::ValueDoesntExist(
							errors::ValueDoesntExist(value)
						)).unwrap_err()
					)
			},
			Term::Expression(value) => self.evaluate_expression(*value, stop_on_ident)
		}
	}

	fn evaluate_function(&self, function: ast::expressions::Function) -> Result<RuntimeValue> {
		let arguments = function
			.arguments
			.into_iter()
			.map(|(argument, _)| argument)
			.collect::<Vec<_>>();

		Ok(RuntimeValue::Function(RuntimeFunction {
			arguments,
			statements: function.statements,

			source: self.source.to_owned(),
			file: self.file.to_owned()
		}))
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

				return self.create_error(entry.position, InterpretErrorKind::DuplicateObjectEntry(
					errors::DuplicateObjectEntry {
						entry_name: name,
						definition_pos
					}
				));
			}

			position_map.insert(name, entry.position);
		}

		Ok(RuntimeValue::Object(value_map))
	}

	fn evaluate_list(&mut self, list: ast::expressions::List) -> Result<RuntimeValue> {
		let mut values = vec![];

		for expression in list.0 {
			let value = self.evaluate_expression(expression, false)?;
			values.push(value);
		}

		Ok(RuntimeValue::List(values))
	}

	fn evaluate_call(&mut self, call: ast::expressions::Call) -> Result<RuntimeValue> {
		use std::iter::zip;

		let call_site = CallSite {
			source: self.source.clone(),
			file: self.file.clone(),
			position: call.position
		};

		let call_arguments_pos = call.arguments.1;
		let call_arguments = call
			.arguments
			.0
			.clone()
			.into_iter()
			.map(|arg| self.evaluate_expression(arg, false))
			.collect::<Result<Vec<_>>>()?;

		let original_expression = *call.function.clone();
		// TODO: store position data in runtime values to be able to retrieve the initial definition
		let function_pos = original_expression.position();

		let expression = self.evaluate_expression(*call.function, false)?;

		if let RuntimeValue::IntrinsicFunction(function) = expression {
			let convert_error = |e: arg_parser::ArgumentParseError| {
				match e {
					arg_parser::ArgumentParseError::CountMismatch {
						expected,
						end_boundary,
						got
					} => self.create_error(
						call_arguments_pos,
						InterpretErrorKind::ArgumentCountMismatch(errors::ArgumentCountMismatch {
							expected, end_boundary, got, function_pos
						})
					).unwrap_err(),

					arg_parser::ArgumentParseError::IncorrectType {
						index,
						expected,
						got
					} => {
						let argument = call
							.arguments
							.0
							.get(index)
							.unwrap_or_else(|| unreachable!("Argument at index `{index}` does not exist when it should"));

						self.create_error(
							argument.position(),
							InterpretErrorKind::ArgumentTypeMismatch(errors::ArgumentTypeMismatch {
								expected, got, function_pos
							})
						).unwrap_err()
					}
				}
			};

			let call_arguments = function
				.arguments
				.verify(&call_arguments)
				.map_err(convert_error)?;
			
			self.context.deeper();
			let result = (function.pointer)(&mut self.context, call_arguments, call_site);
			self.context.shallower();

			return result;
		}

		if let RuntimeValue::Function(function) = expression {
			let got_len = call_arguments.len();
			let expected_len = function.arguments.len();
			
			if got_len != expected_len {
				self.create_error(call_arguments_pos, InterpretErrorKind::ArgumentCountMismatch(
					errors::ArgumentCountMismatch {
						expected: expected_len..expected_len,
						end_boundary: true,
						got: got_len,
						function_pos
					}
				))?;
			}

			let source = self.source.clone();
			let file = self.file.clone();
			
			// Function execution
			let result = {
				self.context.deeper();
				self.source = function.source;
				self.file = function.file;

				for (arg_name, arg_value) in zip(function.arguments, call_arguments) {
					self.context.insert_value(arg_name.clone(), arg_value)?;
				}

				let result = self.execute(ast::Program { statements: function.statements });

				self.context.shallower();
				self.source = source;
				self.file = file;

				result
			};

			result
		} else {
			self.create_error(
				original_expression.position(),
				InterpretErrorKind::ExpressionNotCallable(
					errors::ExpressionNotCallable(expression.kind())
				)
			)
		}
	}
}