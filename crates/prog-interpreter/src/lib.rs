pub mod arg_parser;
pub mod context;
pub mod intrinsics;
pub mod values;

use context::RuntimeContext;
use values::{RuntimeValue, RuntimeFunction};

use prog_parser::ast;
use anyhow::{Result, bail};

#[derive(Debug)]
pub struct Interpreter {
	pub context: RuntimeContext
}

impl Default for Interpreter {
	fn default() -> Self {
		Self::new()
	}
}

impl Interpreter {
	pub fn new() -> Self {
		Self {
			context: RuntimeContext::new()
		}
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
			ast::Statement::VariableDefine { name, value } => self.execute_variable_define(name, value),
			ast::Statement::VariableAssign { name, value } => self.execute_variable_assign(name, value),
			ast::Statement::DoBlock(statements) => self.execute_do_block(statements),

			ast::Statement::Return(expression) => match expression {
				Some(expression) => self.evaluate_expression(expression),
				None => Ok(RuntimeValue::Empty)
			},

			ast::Statement::Call(call) => self.evaluate_call(call),
			ast::Statement::WhileLoop { condition, statements } => self.execute_while_loop(condition, statements),

			ast::Statement::Break => unimplemented!("break"),
			ast::Statement::Continue => unimplemented!("continue"),

			ast::Statement::If { condition, statements, elseif_branches, else_branch } => self.execute_if(condition, statements, elseif_branches, else_branch)
		}
	}

	fn execute_variable_define(&mut self, name: String, value: Option<ast::Expression>) -> Result<RuntimeValue> {
		let evaluated_value = match value {
			None => RuntimeValue::Empty,
			Some(expression) => self.evaluate_expression(expression)?
		};

		self.context.insert_value(name, evaluated_value)?;
		Ok(RuntimeValue::Empty)
	}

	fn execute_variable_assign(&mut self, name: String, value: ast::Expression) -> Result<RuntimeValue> {
		let evaluated_value = self.evaluate_expression(value)?;

		self.context.update_value(name, evaluated_value)?;
		Ok(RuntimeValue::Empty)
	}

	fn execute_do_block(&mut self, statements: Vec<ast::Statement>) -> Result<RuntimeValue> {
		self.context.deeper();
		let result = self.execute(ast::Program { statements });
		self.context.shallower();

		result
	}

	fn execute_while_loop(&mut self, condition: ast::Expression, statements: Vec<ast::Statement>) -> Result<RuntimeValue> {
		let mut evaluated = self.evaluate_expression(condition.clone())?;

		while self.is_value_truthy(&evaluated) {
			self.context.deeper();
			self.execute(ast::Program { statements: statements.clone() })?;
			self.context.shallower();

			evaluated = self.evaluate_expression(condition.clone())?;
		}

		Ok(RuntimeValue::Empty)
	}

	fn execute_if(&mut self, condition: ast::Expression, statements: Vec<ast::Statement>, elseif_branches: Vec<ast::ConditionBranch>, else_branch: Option<ast::ConditionBranch>) -> Result<RuntimeValue> {
		let evaluated = self.evaluate_expression(condition)?;

		if self.is_value_truthy(&evaluated) {
			self.context.deeper();
			self.execute(ast::Program { statements })?;
			self.context.shallower();

			return Ok(RuntimeValue::Empty);
		}

		for branch in elseif_branches {
			let evaluated = self.evaluate_expression(branch.condition)?; 

			if self.is_value_truthy(&evaluated) {
				self.context.deeper();
				self.execute(ast::Program { statements: branch.statements })?;
				self.context.shallower();

				return Ok(RuntimeValue::Empty);
			}
		}

		if let Some(branch) = else_branch {
			let evaluated = self.evaluate_expression(branch.condition)?; 

			if self.is_value_truthy(&evaluated) {
				self.context.deeper();
				self.execute(ast::Program { statements: branch.statements })?;
				self.context.shallower();

				return Ok(RuntimeValue::Empty);
			}
		}

		Ok(RuntimeValue::Empty)
	}

	fn is_value_truthy(&self, rv: &RuntimeValue) -> bool {
		use RuntimeValue as Rv;
		
		match rv {
			Rv::Boolean(value) => *value,
			Rv::String(value) => !value.is_empty(),
			Rv::Number(value) => value != &0.0,
			Rv::List(value) => !value.is_empty(),

			Rv::Function(_) => true,
			Rv::IntrinsicFunction(..) => true,

			Rv::Empty => false
		}
	}

	fn evaluate_expression(&mut self, expression: ast::Expression) -> Result<RuntimeValue> {
		use ast::expressions::*;
		
		match expression {
			Expression::Unary(expression) => self.evaluate_unary_expression(expression.operator, expression.operand),
			Expression::Binary(expression) => self.evaluate_binary_expression(expression.lhs, expression.operator, expression.rhs),
			Expression::Term(term) => self.evaluate_term(term),
			Expression::Empty => Ok(RuntimeValue::Empty)
		}
	}

	fn evaluate_unary_expression(&mut self, operator: ast::expressions::operators::UnaryOperator, operand: ast::expressions::Term) -> Result<RuntimeValue> {
		use ast::expressions::operators::UnaryOperator as Op;
		use RuntimeValue as Rv;

		let evaluated_operand = self.evaluate_term(operand)?;

		match (operator, evaluated_operand) {
			(Op::Minus, Rv::Number(value)) => Ok(Rv::Number(-value)),

			(Op::Not, Rv::Boolean(value)) => Ok(Rv::Boolean(!value)),
			(Op::Not, Rv::String(value)) => Ok(Rv::Boolean(value.is_empty())),
			(Op::Not, Rv::Number(value)) => Ok(Rv::Boolean(value == 0.0)),
			(Op::Not, Rv::List(value)) => Ok(Rv::Boolean(value.is_empty())),
			(Op::Not, Rv::Function(_)) => Ok(Rv::Boolean(false)),
			(Op::Not, Rv::IntrinsicFunction(..)) => Ok(Rv::Boolean(false)),
			(Op::Not, Rv::Empty) => Ok(Rv::Boolean(true)),

		 	(operator, operand) => bail!("Cannot perform an unsupported unary operation '{}' on '{}'", operator, operand)
		}
	}

	fn evaluate_binary_expression(&mut self, lhs: ast::expressions::Term, operator: ast::expressions::operators::BinaryOperator, rhs: ast::expressions::Term) -> Result<RuntimeValue> {
		use ast::expressions::operators::BinaryOperator as Op;
		use RuntimeValue as Rv;
		
		let evaluated_lhs = self.evaluate_term(lhs)?;
		let evaluated_rhs = self.evaluate_term(rhs)?;

		match (operator, evaluated_lhs, evaluated_rhs) {
			(Op::Plus, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Number(lhs + rhs)),
			(Op::Minus, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Number(lhs - rhs)),
			(Op::Divide, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Number(lhs / rhs)),
			(Op::Multiply, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Number(lhs * rhs)),
			(Op::Modulo, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Number(lhs % rhs)),
			(Op::Gt, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Boolean(lhs > rhs)),
			(Op::Lt, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Boolean(lhs < rhs)),
			(Op::Gte, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Boolean(lhs >= rhs)),
			(Op::Lte, Rv::Number(lhs), Rv::Number(rhs)) => Ok(Rv::Boolean(lhs <= rhs)),

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

			(Op::EqEq, _, _) => Ok(Rv::Boolean(false)),
			(Op::NotEq, _, _) => Ok(Rv::Boolean(true)),

			(operator, lhs, rhs) => bail!("Cannot perform an unsupported binary operation '{}' on '{}' and '{}'", operator, lhs, rhs)
		}
	}

	fn evaluate_term(&mut self, term: ast::expressions::Term) -> Result<RuntimeValue> {
		use ast::expressions::*;

		match term {
			Term::List(value) => self.evaluate_list(value),
			Term::Call(value) => self.evaluate_call(value),
			Term::Function(value) => self.evaluate_function(value),
			Term::Literal(value) => Ok(value.into()),
			Term::Identifier(value) => self.context.get_value(&value),
			Term::Expression(value) => self.evaluate_expression(*value)
		}
	}

	fn evaluate_function(&self, function: ast::expressions::Function) -> Result<RuntimeValue> {
		Ok(RuntimeValue::Function(RuntimeFunction {
			arguments: function.arguments,
			statements: function.statements
		}))
	}

	fn evaluate_list(&mut self, list: ast::expressions::List) -> Result<RuntimeValue> {
		let mut values = vec![];

		for expression in list.0 {
			let value = self.evaluate_expression(expression)?;
			values.push(value);
		}

		Ok(RuntimeValue::List(values))
	}

	fn evaluate_call(&mut self, call: ast::expressions::Call) -> Result<RuntimeValue> {
		use std::iter::zip;

		let call_arguments = call
			.arguments
			.into_iter()
			.map(|arg| self.evaluate_expression(arg))
			.collect::<Result<Vec<RuntimeValue>>>()?;

		let original_expression = *call.function.clone();
		let expression = self.evaluate_expression(*call.function)?;

		if let RuntimeValue::IntrinsicFunction(function) = expression {
			let call_arguments = function.arguments.verify(&call_arguments)?;
			
			self.context.deeper();
			let result = (function.pointer)(&mut self.context, call_arguments);
			self.context.shallower();

			return result;
		}

		if let RuntimeValue::Function(function) = expression {
			let expected_str = format!("(expected {} arguments, got {})", function.arguments.len(), call_arguments.len());

			let got_len = call_arguments.len();
			let expected_len = function.arguments.len();
			
			if got_len != expected_len {
				let word = if got_len < expected_len { "few" } else { "many" };
				bail!("Too {} arguments in function call {}", word, expected_str);
			}
			
			self.context.deeper();
			for (index, (arg_name, arg_value)) in zip(function.arguments, call_arguments).enumerate() {
				if self.context.key_real(&arg_name) {
					bail!("Name of argument #{} conflicts with variable name '{}'", index + 1, arg_name);
				}

				self.context.insert_value(arg_name, arg_value)?;
			}

			let result = self.execute(ast::Program { statements: function.statements });
			self.context.shallower();

			result
		} else {
			bail!("Expression '{:?}' is not callable", original_expression);
		}
	}
}