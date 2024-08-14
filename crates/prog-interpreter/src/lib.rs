pub mod arg_parser;
pub mod context;
pub mod errors;
pub mod intrinsics;
pub mod values;

use anyhow::Result;
use context::RuntimeContext;
pub use errors::{InterpretError, InterpretErrorKind};
use halloc::Memory;
use prog_parser::ast;
use values::{CallSite, RuntimeFunction};
pub use values::{RuntimePrimitive, RuntimeValue, RuntimeValueKind};

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
	pub context: RuntimeContext,

	source: String,
	file: String
}

impl Interpreter {
	pub fn new(source: String, file: String) -> Self {
		Self {
			memory: Memory::new(),
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

		self.context.insert(statement.name.0, evaluated_value);
		Ok(RuntimeValue::Empty)
	}

	fn execute_variable_assign(&mut self, statement: ast::VariableAssign) -> Result<RuntimeValue> {
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

		Ok(RuntimeValue::Empty)
	}

	fn execute_do_block(&mut self, statement: ast::DoBlock) -> Result<RuntimeValue> {
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

	fn execute_return(&mut self, statement: ast::Return) -> Result<RuntimeValue> {
		let value = match statement.expression {
			Some(expression) => self.evaluate_expression(expression, false)?,
			None => RuntimeValue::Empty
		};

		Ok(RuntimeValue::Marker(values::MarkerKind::Return(Box::new(
			value
		))))
	}

	fn execute_while_loop(&mut self, statement: ast::WhileLoop) -> Result<RuntimeValue> {
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

		if evaluated.is_truthy() {
			self.context.deeper();
			let result = self.execute(
				ast::Program {
					statements: statement.statements
				},
				true
			)?;
			self.context.shallower();

			if result.kind() == RuntimeValueKind::Marker {
				return Ok(result);
			}

			return Ok(RuntimeValue::Empty);
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

				if result.kind() == RuntimeValueKind::Marker {
					return Ok(result);
				}

				return Ok(RuntimeValue::Empty);
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

			if result.kind() == RuntimeValueKind::Marker {
				return Ok(result);
			}

			return Ok(RuntimeValue::Empty);
		}

		Ok(RuntimeValue::Empty)
	}

	fn execute_expression_assign(
		&mut self,
		statement: ast::ExpressionAssign
	) -> Result<RuntimeValue> {
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
		value: RuntimeValue
	) -> Result<RuntimeValue> {
		let ast::expressions::Binary {
			lhs,
			rhs,
			operator: _,
			position
		} = expression;

		let list_name = match self.evaluate_term(rhs.clone(), true)? {
			RuntimeValue::Identifier(identifier) => identifier,
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
			RuntimeValue::Number(index) => index.owned() as i64,
			value => {
				create_error!(
					self,
					position,
					InterpretErrorKind::CannotIndexValue(errors::CannotIndexValue {
						kind: (RuntimeValueKind::List, rhs.position()),
						expected_index_kind: RuntimeValueKind::Number,
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
					kind: (RuntimeValueKind::List, rhs.position()),
					expected_index_kind: RuntimeValueKind::Number,
					index_kind: (value.kind(), lhs.position()),
					because_negative: true
				})
			);
		}

		let index: usize = index.try_into()?;
		let inner_list = match self.context.get_mut(&list_name)? {
			RuntimeValue::List(inner_list) => inner_list,
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

		if index >= inner_list.value().len() {
			inner_list.0.resize(index + 1, RuntimeValue::Empty);
		}

		inner_list.0[index] = value;

		Ok(RuntimeValue::Empty)
	}

	fn execute_expression_assign_object(
		&mut self,
		expression: ast::expressions::Binary,
		value: RuntimeValue
	) -> Result<RuntimeValue> {
		let ast::expressions::Binary {
			lhs,
			rhs,
			operator: _,
			position
		} = expression.clone();

		let object_name = match self.evaluate_term(lhs.clone(), true)? {
			RuntimeValue::Identifier(identifier) => identifier,
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
			RuntimeValue::Identifier(value) => value,
			RuntimeValue::String(value) => value.owned(),

			value => {
				create_error!(
					self,
					position,
					InterpretErrorKind::CannotIndexValue(errors::CannotIndexValue {
						kind: (RuntimeValueKind::Object, lhs.position()),
						expected_index_kind: RuntimeValueKind::String,
						index_kind: (value.kind(), rhs.position()),
						because_negative: false
					})
				)
			}
		};

		let inner_object = match self.context.get_mut(&object_name)? {
			RuntimeValue::Object(inner_object) => inner_object,

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
		let mut map = inner_object.0.get().to_owned();
		map.insert(entry_name, value);

		inner_object.0.write(map);

		Ok(RuntimeValue::Empty)
	}

	fn evaluate_expression(
		&mut self,
		expression: ast::Expression,
		stop_on_ident: bool
	) -> Result<RuntimeValue> {
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
			(Op::Minus, Rv::Number(v)) => Ok(Rv::Number((-v.owned()).into())),

			(Op::Not, Rv::Boolean(v)) => Ok(Rv::Boolean((!v.value()).into())),
			(Op::Not, Rv::String(v)) => Ok(Rv::Boolean(v.value().is_empty().into())),
			(Op::Not, Rv::Number(v)) => Ok(Rv::Boolean((v.owned() == 0.0).into())),
			(Op::Not, Rv::List(v)) => Ok(Rv::Boolean(v.value().is_empty().into())),
			(Op::Not, Rv::Function(_)) => Ok(Rv::Boolean(false.into())),
			(Op::Not, Rv::IntrinsicFunction(..)) => Ok(Rv::Boolean(false.into())),
			(Op::Not, Rv::Empty) => Ok(Rv::Boolean(true.into())),

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

				RuntimeValue::IntrinsicFunction(function.into())
			}};
		}

		let evaluated_expr = match (operator.0, evaluated_lhs, evaluated_rhs) {
			(Op::Add, Rv::Number(lhs), Rv::Number(rhs)) => {
				Rv::Number((lhs.owned() + rhs.owned()).into())
			}
			(Op::Subtract, Rv::Number(lhs), Rv::Number(rhs)) => {
				Rv::Number((lhs.owned() - rhs.owned()).into())
			}
			(Op::Divide, Rv::Number(lhs), Rv::Number(rhs)) => {
				Rv::Number((lhs.owned() / rhs.owned()).into())
			}
			(Op::Multiply, Rv::Number(lhs), Rv::Number(rhs)) => {
				Rv::Number((lhs.owned() * rhs.owned()).into())
			}
			(Op::Modulo, Rv::Number(lhs), Rv::Number(rhs)) => {
				Rv::Number((lhs.owned() % rhs.owned()).into())
			}
			(Op::Gt, Rv::Number(lhs), Rv::Number(rhs)) => {
				Rv::Boolean((lhs.owned() > rhs.owned()).into())
			}
			(Op::Lt, Rv::Number(lhs), Rv::Number(rhs)) => {
				Rv::Boolean((lhs.owned() < rhs.owned()).into())
			}
			(Op::Gte, Rv::Number(lhs), Rv::Number(rhs)) => {
				Rv::Boolean((lhs.owned() >= rhs.owned()).into())
			}
			(Op::Lte, Rv::Number(lhs), Rv::Number(rhs)) => {
				Rv::Boolean((lhs.owned() <= rhs.owned()).into())
			}

			(Op::Add, Rv::String(lhs), rhs) => Rv::String(format!("{}{}", lhs.value(), rhs).into()),

			(Op::And, Rv::Boolean(lhs), Rv::Boolean(rhs)) => {
				Rv::Boolean((lhs.owned() && rhs.owned()).into())
			}
			(Op::Or, Rv::Boolean(lhs), Rv::Boolean(rhs)) => {
				Rv::Boolean((lhs.owned() || rhs.owned()).into())
			}

			(Op::EqEq, Rv::Boolean(lhs), Rv::Boolean(rhs)) => Rv::Boolean((lhs == rhs).into()),
			(Op::EqEq, Rv::String(lhs), Rv::String(rhs)) => Rv::Boolean((lhs == rhs).into()),
			(Op::EqEq, Rv::Number(lhs), Rv::Number(rhs)) => Rv::Boolean((lhs == rhs).into()),
			(Op::EqEq, Rv::List(lhs), Rv::List(rhs)) => Rv::Boolean((lhs == rhs).into()),
			(Op::EqEq, Rv::Object(lhs), Rv::Object(rhs)) => Rv::Boolean((lhs == rhs).into()),
			(Op::EqEq, Rv::Function(lhs), Rv::Function(rhs)) => Rv::Boolean((lhs == rhs).into()),
			(Op::EqEq, Rv::IntrinsicFunction(lhs), Rv::IntrinsicFunction(rhs)) => {
				Rv::Boolean((lhs == rhs).into())
			}
			(Op::EqEq, Rv::Empty, Rv::Empty) => Rv::Boolean(true.into()),

			(Op::NotEq, Rv::Boolean(lhs), Rv::Boolean(rhs)) => Rv::Boolean((lhs != rhs).into()),
			(Op::NotEq, Rv::String(lhs), Rv::String(rhs)) => Rv::Boolean((lhs != rhs).into()),
			(Op::NotEq, Rv::Number(lhs), Rv::Number(rhs)) => Rv::Boolean((lhs != rhs).into()),
			(Op::NotEq, Rv::List(lhs), Rv::List(rhs)) => Rv::Boolean((lhs != rhs).into()),
			(Op::NotEq, Rv::Object(lhs), Rv::Object(rhs)) => Rv::Boolean((lhs != rhs).into()),
			(Op::NotEq, Rv::Function(lhs), Rv::Function(rhs)) => Rv::Boolean((lhs != rhs).into()),
			(Op::NotEq, Rv::IntrinsicFunction(lhs), Rv::IntrinsicFunction(rhs)) => {
				Rv::Boolean((lhs != rhs).into())
			}
			(Op::NotEq, Rv::Empty, Rv::Empty) => Rv::Boolean(false.into()),

			(Op::ListAccess, Rv::Number(lhs), Rv::List(rhs)) => {
				rhs.value()
					.get(lhs.owned() as usize)
					.cloned()
					.unwrap_or(RuntimeValue::Empty)
			}

			(Op::ObjectAccess, Rv::Object(lhs), Rv::Identifier(rhs)) => {
				lhs.value()
					.get(&rhs)
					.cloned()
					.unwrap_or(RuntimeValue::Empty)
			}
			(Op::ObjectAccess, Rv::Object(lhs), Rv::String(rhs)) => {
				lhs.value()
					.get(rhs.value())
					.cloned()
					.unwrap_or(RuntimeValue::Empty)
			}

			(Op::ObjectAccess, Rv::Boolean(lhs), Rv::Identifier(rhs)) => {
				primitive_object_access!(lhs, rhs)
			}
			(Op::ObjectAccess, Rv::String(lhs), Rv::Identifier(rhs)) => {
				primitive_object_access!(lhs, rhs)
			}
			(Op::ObjectAccess, Rv::Number(lhs), Rv::Identifier(rhs)) => {
				primitive_object_access!(lhs, rhs)
			}
			(Op::ObjectAccess, Rv::List(lhs), Rv::Identifier(rhs)) => {
				primitive_object_access!(lhs, rhs)
			}

			(Op::ObjectAccess, Rv::Boolean(lhs), Rv::String(rhs)) => {
				primitive_object_access!(lhs, rhs.owned())
			}
			(Op::ObjectAccess, Rv::String(lhs), Rv::String(rhs)) => {
				primitive_object_access!(lhs, rhs.owned())
			}
			(Op::ObjectAccess, Rv::Number(lhs), Rv::String(rhs)) => {
				primitive_object_access!(lhs, rhs.owned())
			}
			(Op::ObjectAccess, Rv::List(lhs), Rv::String(rhs)) => {
				primitive_object_access!(lhs, rhs.owned())
			}

			(Op::EqEq, _, _) => Rv::Boolean(false.into()),
			(Op::NotEq, _, _) => Rv::Boolean(true.into()),

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
	) -> Result<RuntimeValue> {
		use ast::expressions::*;

		let position = term.position();

		match term {
			Term::Object(obj) => self.evaluate_object(obj),
			Term::List(list) => self.evaluate_list(list),
			Term::Call(call) => self.evaluate_call(call),
			Term::Function(func) => self.evaluate_function(func),
			Term::Literal(lit) => Ok(lit.into()),
			Term::Identifier(ident, _) => {
				match stop_on_ident {
					true => Ok(RuntimeValue::Identifier(ident)),
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

	fn evaluate_function(&self, function: ast::expressions::Function) -> Result<RuntimeValue> {
		let converted = RuntimeFunction {
			ast: Box::new(function),
			source: self.source.to_owned(),
			file: self.file.to_owned()
		};

		Ok(RuntimeValue::Function(converted))
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
		Ok(RuntimeValue::Object(allocated.into()))
	}

	fn evaluate_list(&mut self, list: ast::expressions::List) -> Result<RuntimeValue> {
		let mut values = vec![];

		for expression in list.0 {
			let value = self.evaluate_expression(expression, false)?;
			values.push(value);
		}

		Ok(RuntimeValue::List(values.into()))
	}

	fn evaluate_call(&mut self, call: ast::expressions::Call) -> Result<RuntimeValue> {
		let call_site = CallSite {
			source: self.source.clone(),
			file: self.file.clone(),

			args_pos: call.arguments.1,
			func_pos: call.function.position(),
			whole_pos: call.position
		};

		let call_args_pos = call_site.args_pos.clone();
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

		if let RuntimeValue::IntrinsicFunction(function) = function_expr {
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
							unreachable!(
								"Argument at index `{index}` does not exist when it should"
							)
						});

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

			let call_args = function
				.arguments
				.verify(&call_args)
				.map_err(convert_error)?;

			self.context.deeper();
			let result = function.call(self, call_args, call_site);
			self.context.shallower();

			return result;
		}

		if let RuntimeValue::Function(function) = function_expr {
			let got_len = call_args.len();
			let expected_len = function.ast.arguments.len();

			if got_len != expected_len {
				let first_arg = function.ast.arguments.first().unwrap();

				let last_arg = function.ast.arguments.last().unwrap();

				let fn_call_pos = function_pos;
				let fn_def_args_pos = Some((first_arg.1.start)..(last_arg.1.end));

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
				self.context.deeper();
				self.source = function.source;
				self.file = function.file;

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

				self.context.shallower();
				self.source = source;
				self.file = file;

				result
			}
		} else {
			create_error!(
				self,
				original_expr.position(),
				InterpretErrorKind::ExpressionNotCallable(errors::ExpressionNotCallable(
					function_expr.kind()
				))
			)
		}
	}
}
