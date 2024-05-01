pub mod arg_parser;
pub mod context;
pub mod intrinsics;
pub mod values;

use context::RuntimeContext;
use values::{RuntimeFunction, RuntimeValue, RuntimeValueKind};

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
				Some(expression) => self.evaluate_expression(expression, false),
				None => Ok(RuntimeValue::Empty)
			},

			ast::Statement::Call(call) => self.evaluate_call(call),
			ast::Statement::WhileLoop { condition, statements } => self.execute_while_loop(condition, statements),

			ast::Statement::Break => unimplemented!("break"),
			ast::Statement::Continue => unimplemented!("continue"),

			ast::Statement::If { condition, statements, elseif_branches, else_branch } => self.execute_if(condition, statements, elseif_branches, else_branch),
		
			ast::Statement::ExpressionAssign { expression, value } => self.execute_expression_assign(expression, value)
		}
	}

	fn execute_variable_define(&mut self, name: String, value: Option<ast::Expression>) -> Result<RuntimeValue> {
		let evaluated_value = match value {
			None => RuntimeValue::Empty,
			Some(expression) => self.evaluate_expression(expression, false)?
		};

		self.context.insert_value(name, evaluated_value)?;
		Ok(RuntimeValue::Empty)
	}

	fn execute_variable_assign(&mut self, name: String, value: ast::Expression) -> Result<RuntimeValue> {
		let evaluated_value = self.evaluate_expression(value, false)?;

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
		let mut evaluated = self.evaluate_expression(condition.clone(), false)?;

		while self.is_value_truthy(&evaluated) {
			self.context.deeper();
			self.execute(ast::Program { statements: statements.clone() })?;
			self.context.shallower();

			evaluated = self.evaluate_expression(condition.clone(), false)?;
		}

		Ok(RuntimeValue::Empty)
	}

	fn execute_if(&mut self, condition: ast::Expression, statements: Vec<ast::Statement>, elseif_branches: Vec<ast::ConditionBranch>, else_branch: Option<ast::ConditionBranch>) -> Result<RuntimeValue> {
		let evaluated = self.evaluate_expression(condition, false)?;

		if self.is_value_truthy(&evaluated) {
			self.context.deeper();
			self.execute(ast::Program { statements })?;
			self.context.shallower();

			return Ok(RuntimeValue::Empty);
		}

		for branch in elseif_branches {
			let evaluated = self.evaluate_expression(branch.condition, false)?; 

			if self.is_value_truthy(&evaluated) {
				self.context.deeper();
				self.execute(ast::Program { statements: branch.statements })?;
				self.context.shallower();

				return Ok(RuntimeValue::Empty);
			}
		}

		if let Some(branch) = else_branch {
			let evaluated = self.evaluate_expression(branch.condition, false)?; 

			if self.is_value_truthy(&evaluated) {
				self.context.deeper();
				self.execute(ast::Program { statements: branch.statements })?;
				self.context.shallower();

				return Ok(RuntimeValue::Empty);
			}
		}

		Ok(RuntimeValue::Empty)
	}

	fn execute_expression_assign(&mut self, expression: ast::Expression, value: ast::Expression) -> Result<RuntimeValue> {
		use ast::expressions::operators::BinaryOperator as Op;

		let expression = match expression {
			ast::Expression::Binary(expression) => expression,
			_ => bail!("Expression `{:?}` is not assignable", expression)
		};

		if !matches!(expression.operator, Op::ListAccess | Op::ObjectAccess) {
			bail!("Expression `{:?}` is not assignable", expression);
		}

		let value = self.evaluate_expression(value, false)?;

		if expression.operator == Op::ListAccess {
			self.execute_expression_assign_list(expression, value)
		} else {
			self.execute_expression_assign_object(expression, value)
		}
	}

	fn execute_expression_assign_list(&mut self, expression: ast::expressions::Binary, value: RuntimeValue) -> Result<RuntimeValue> {
		let ast::expressions::Binary { lhs, rhs, operator: _ } = expression.clone();

		let list_name = match self.evaluate_term(rhs, true)? {
			RuntimeValue::Identifier(identifier) => identifier.0,
			_ => bail!("Expression `{expression}` is not assignable")
		};

		let mut inner_list = match self.context.get_value(&list_name)? {
			RuntimeValue::List(inner_list) => inner_list,
			_ => bail!("Expression `{expression}` is not assignable")
		};

		let index = match self.evaluate_term(lhs, false)? {
			RuntimeValue::Number(index) => index as i64,
			value => bail!(
				"Cannot index `{}` using `{}`",
				RuntimeValueKind::List,
				value.kind()
			)
		};

		if index.is_negative() {
			bail!("Value `{index}` cannot be used to index `{list_name}` as it is negative");
		}

		let index: usize = index.try_into()?;

		if index >= inner_list.len() {
			inner_list.resize(index + 50, RuntimeValue::Empty);
		}

		inner_list[index] = value;

		self.context.update_value(
			list_name,
			RuntimeValue::List(inner_list)
		)?;

		Ok(RuntimeValue::Empty)
	}

	fn execute_expression_assign_object(&mut self, expression: ast::expressions::Binary, value: RuntimeValue) -> Result<RuntimeValue> {
		let ast::expressions::Binary { lhs, rhs, operator: _ } = expression.clone();

		let object_name = match self.evaluate_term(lhs, true)? {
			RuntimeValue::Identifier(identifier) => identifier.0,
			_ => bail!("Expression `{expression}` is not assignable")
		};

		let mut inner_object = match self.context.get_value(&object_name)? {
			RuntimeValue::Object(inner_object) => inner_object,
			_ => bail!("Expression `{expression}` is not assignable")
		};

		let entry_name = match self.evaluate_term(rhs, true)? {
			RuntimeValue::Identifier(value) => value.0,
			RuntimeValue::String(value) => value,

			value => bail!(
				"Cannot index `{}` using `{}`",
				RuntimeValueKind::Object,
				value.kind()
			)
		};

		inner_object.insert(entry_name, value);

		self.context.update_value(
			object_name,
			RuntimeValue::Object(inner_object)
		)?;

		Ok(RuntimeValue::Empty)
	}

	fn is_value_truthy(&self, rv: &RuntimeValue) -> bool {
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

	fn identifier_from_term(&self, term: &ast::expressions::Term) -> Option<String> {
		match term {
			ast::expressions::Term::Identifier(value) => Some(value.to_owned()),
			ast::expressions::Term::Expression(value) => {
				if let ast::expressions::Expression::Term(ref value) = value.as_ref() {
					self.identifier_from_term(value)
				} else {
					None
				}
			},

			_ => None
		}
	}

	fn evaluate_expression(&mut self, expression: ast::Expression, stop_on_ident: bool) -> Result<RuntimeValue> {
		use ast::expressions::*;
		
		match expression {
			Expression::Unary(expression) => self.evaluate_unary_expression(expression.operator, expression.operand, stop_on_ident),
			Expression::Binary(expression) => self.evaluate_binary_expression(expression.lhs, expression.operator, expression.rhs, stop_on_ident),
			Expression::Term(term) => self.evaluate_term(term, stop_on_ident),
			Expression::Empty => Ok(RuntimeValue::Empty)
		}
	}

	fn evaluate_unary_expression(
		&mut self,
		operator: ast::expressions::operators::UnaryOperator,
		operand: ast::expressions::Term,
		stop_on_ident: bool
	) -> Result<RuntimeValue> {
		use ast::expressions::operators::UnaryOperator as Op;
		use RuntimeValue as Rv;

		let evaluated_operand = self.evaluate_term(operand, stop_on_ident)?;

		match (operator, evaluated_operand) {
			(Op::Minus, Rv::Number(value)) => Ok(Rv::Number(-value)),

			(Op::Not, Rv::Boolean(value)) => Ok(Rv::Boolean(!value)),
			(Op::Not, Rv::String(value)) => Ok(Rv::Boolean(value.is_empty())),
			(Op::Not, Rv::Number(value)) => Ok(Rv::Boolean(value == 0.0)),
			(Op::Not, Rv::List(value)) => Ok(Rv::Boolean(value.is_empty())),
			(Op::Not, Rv::Function(_)) => Ok(Rv::Boolean(false)),
			(Op::Not, Rv::IntrinsicFunction(..)) => Ok(Rv::Boolean(false)),
			(Op::Not, Rv::Empty) => Ok(Rv::Boolean(true)),

		 	(operator, operand) => bail!("Cannot perform an unsupported unary operation `{}` on `{}`", operator, operand)
		}
	}

	fn evaluate_binary_expression(
		&mut self,
		lhs: ast::expressions::Term,
		operator: ast::expressions::operators::BinaryOperator,
		rhs: ast::expressions::Term,
		stop_on_ident: bool
	) -> Result<RuntimeValue> {
		use ast::expressions::operators::BinaryOperator as Op;
		use RuntimeValue as Rv;
		
		let mut evaluated_lhs = self.evaluate_term(lhs.clone(), stop_on_ident)?;
		let evaluated_lhs_forced = self.evaluate_term(lhs, false)?;

		let is_object_access = match (&evaluated_lhs_forced, operator) {
			(RuntimeValue::Object(_), Op::ObjectAccess) => true,
			_ => false
		};

		let evaluated_rhs = match (is_object_access, self.identifier_from_term(&rhs)) {
			(true, Some(value)) => RuntimeValue::Identifier(values::Identifier(value)),
			_ => self.evaluate_term(rhs, false)?
		};

		if is_object_access {
			evaluated_lhs = evaluated_lhs_forced;
		}

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

			(Op::Plus, Rv::String(lhs), rhs) => Ok(Rv::String(format!("{}{}", lhs, rhs.to_string()))),

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

			(operator, lhs, rhs) => bail!("Cannot perform an unsupported binary operation `{} {} {}`", lhs.kind(), operator, rhs.kind())
		}
	}

	fn evaluate_term(&mut self, term: ast::expressions::Term, stop_on_ident: bool) -> Result<RuntimeValue> {
		use ast::expressions::*;

		match term {
			Term::Object(value) => self.evaluate_object(value),
			Term::List(value) => self.evaluate_list(value),
			Term::Call(value) => self.evaluate_call(value),
			Term::Function(value) => self.evaluate_function(value),
			Term::Literal(value) => Ok(value.into()),
			Term::Identifier(value) => match stop_on_ident {
				true => Ok(RuntimeValue::Identifier(value.into())),
				false => self.context.get_value(&value)
			},
			Term::Expression(value) => self.evaluate_expression(*value, stop_on_ident)
		}
	}

	fn evaluate_function(&self, function: ast::expressions::Function) -> Result<RuntimeValue> {
		Ok(RuntimeValue::Function(RuntimeFunction {
			arguments: function.arguments,
			statements: function.statements
		}))
	}

	fn evaluate_object(&mut self, object: ast::expressions::Object) -> Result<RuntimeValue> {
		use std::collections::HashMap;
		let mut new_map = HashMap::new();

		for (name, value) in object.0 {
			let value = self.evaluate_expression(value, false)?;
			
			if new_map.insert(name.clone(), value).is_some() {
				bail!("Duplicate key `{name}` found in object");
			}
		}

		Ok(RuntimeValue::Object(new_map))
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

		let call_arguments = call
			.arguments
			.into_iter()
			.map(|arg| self.evaluate_expression(arg, false))
			.collect::<Result<Vec<RuntimeValue>>>()?;

		let original_expression = *call.function.clone();
		let expression = self.evaluate_expression(*call.function, false)?;

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
			for (arg_name, arg_value) in zip(function.arguments, call_arguments) {
				self.context.insert_value(arg_name, arg_value)?;
			}

			let result = self.execute(ast::Program { statements: function.statements });
			self.context.shallower();

			result
		} else {
			bail!("Expression `{:?}` is not callable", original_expression);
		}
	}
}