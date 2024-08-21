use anyhow::Result;
use prog_parser::ast;

use crate::*;

#[derive(Debug)]
pub struct Compiler {
	label_counter: usize
}

impl Compiler {
	pub fn new() -> Self { Self { label_counter: 0 } }

	pub fn compile(&mut self, ast: ast::Program) -> Result<Vec<Instruction>> {
		let mut emitted = vec![];

		for statement in ast.statements {
			let instructions = self.compile_statement(statement)?;
			emitted.extend(instructions);
		}

		Ok(emitted)
	}

	fn current_label(&self) -> String { format!("L{}", self.label_counter) }

	fn new_label(&mut self) -> String {
		self.label_counter += 1;
		self.current_label()
	}

	fn compile_statements(&mut self, statements: Vec<ast::Statement>) -> Result<Vec<Instruction>> {
		Ok(statements
			.into_iter()
			.map(|stmt| self.compile_statement(stmt))
			.collect::<Result<Vec<_>>>()?
			.into_iter()
			.flatten()
			.collect::<Vec<_>>())
	}

	fn compile_statement(&mut self, statement: ast::Statement) -> Result<Vec<Instruction>> {
		match statement {
			ast::Statement::VariableDefine(stmt) => self.compile_variable_define(stmt),
			ast::Statement::VariableAssign(stmt) => self.compile_variable_assign(stmt),
			ast::Statement::Return(stmt) => self.compile_return(stmt),
			ast::Statement::Call(expr) => self.compile_call(expr),
			ast::Statement::WhileLoop(stmt) => self.compile_while(stmt),
			ast::Statement::If(stmt) => self.compile_if(stmt),

			_ => todo!()
		}
	}

	fn compile_expression(&mut self, expression: ast::Expression) -> Result<Vec<Instruction>> {
		match expression {
			ast::Expression::Unary(expr) => self.compile_unary_expr(expr),
			ast::Expression::Binary(expr) => self.compile_binary_expr(expr),
			ast::Expression::Term(expr) => self.compile_term(expr),
			ast::Expression::Empty(_) => todo!()
		}
	}

	fn compile_unary_expr(
		&mut self,
		expression: ast::expressions::Unary
	) -> Result<Vec<Instruction>> {
		use ast::expressions::operators::UnaryOperator as Op;

		let mut emitted = vec![];
		emitted.extend(self.compile_term(expression.operand)?);

		match expression.operator.0 {
			Op::Minus => emitted.push(Instruction::NEG(NEG)),
			Op::Not => emitted.push(Instruction::NOT(NOT))
		}

		Ok(emitted)
	}

	fn compile_binary_expr(
		&mut self,
		expression: ast::expressions::Binary
	) -> Result<Vec<Instruction>> {
		use ast::expressions::operators::BinaryOperator as Op;

		let mut emitted = vec![];

		emitted.extend(self.compile_term(expression.lhs)?);
		emitted.extend(self.compile_term(expression.rhs)?);

		match expression.operator.0 {
			Op::Add => emitted.push(Instruction::ADD(ADD)),
			Op::Subtract => emitted.push(Instruction::SUB(SUB)),
			Op::Multiply => emitted.push(Instruction::MUL(MUL)),
			Op::Divide => emitted.push(Instruction::DIV(DIV)),
			Op::EqEq => emitted.push(Instruction::EQ(EQ)),
			Op::Gt => emitted.push(Instruction::GT(GT)),
			Op::Lt => emitted.push(Instruction::LT(LT)),
			Op::Gte => emitted.push(Instruction::GTE(GTE)),
			Op::Lte => emitted.push(Instruction::LTE(LTE)),

			_ => todo!()
		}

		Ok(emitted)
	}

	fn compile_term(&mut self, term: ast::expressions::Term) -> Result<Vec<Instruction>> {
		use ast::expressions::Term as T;

		match term {
			T::Call(expr) => self.compile_call(expr),
			T::Function(func) => self.compile_function(func),
			T::Literal(lit) => self.compile_literal(lit),
			T::Identifier(ident, _) => Ok(vec![Instruction::LOAD(LOAD(ident))]),
			T::Expression(expr) => self.compile_expression(*expr),

			_ => todo!()
		}
	}

	fn compile_function(
		&mut self,
		function: ast::expressions::Function
	) -> Result<Vec<Instruction>> {
		let argc = function.arguments.len();

		let mut func_instructions = vec![];

		for (arg, _) in function.arguments {
			func_instructions.push(Instruction::STORE(STORE(arg)));
		}

		for statement in function.statements {
			let stmt_instructions = self.compile_statement(statement)?;
			func_instructions.extend(stmt_instructions);
		}

		Ok(vec![Instruction::NEWFUNC(NEWFUNC(argc, func_instructions))])
	}

	fn compile_literal(&mut self, literal: ast::expressions::Literal) -> Result<Vec<Instruction>> {
		use ast::expressions::Literal as L;

		let value = match literal {
			L::Boolean(lit, _) => Value::Boolean(lit),
			L::String(lit, _) => Value::String(lit),
			L::Number(lit, _) => Value::Number(lit)
		};

		Ok(vec![Instruction::PUSH(PUSH(value))])
	}

	fn compile_variable_define(
		&mut self,
		statement: ast::VariableDefine
	) -> Result<Vec<Instruction>> {
		let name = statement.name.0;
		let mut emitted = statement
			.value
			.map(|expr| self.compile_expression(expr))
			.unwrap_or(Ok(vec![Instruction::PUSH(PUSH(Value::Empty))]))?;

		emitted.push(Instruction::STORE(STORE(name)));

		Ok(emitted)
	}

	fn compile_variable_assign(
		&mut self,
		statement: ast::VariableAssign
	) -> Result<Vec<Instruction>> {
		let name = statement.name.0;
		let mut emitted = self.compile_expression(statement.value)?;

		emitted.push(Instruction::STORE(STORE(name)));

		Ok(emitted)
	}

	fn compile_return(&mut self, statement: ast::Return) -> Result<Vec<Instruction>> {
		let mut emitted = statement
			.expression
			.map(|expr| self.compile_expression(expr))
			.unwrap_or(Ok(vec![Instruction::PUSH(PUSH(Value::Empty))]))?;

		emitted.push(Instruction::RET(RET));
		Ok(emitted)
	}

	fn compile_call(&mut self, expression: ast::expressions::Call) -> Result<Vec<Instruction>> {
		let mut emitted = vec![];

		for arg in expression.arguments.0 {
			let arg_instructions = self.compile_expression(arg)?;
			emitted.extend(arg_instructions);
		}

		emitted.extend(self.compile_expression(*expression.function)?);

		emitted.push(Instruction::CALL(CALL));
		Ok(emitted)
	}

	fn compile_while(&mut self, statement: ast::WhileLoop) -> Result<Vec<Instruction>> {
		let mut emitted = vec![];

		let condition_label = self.new_label();
		let body_label = self.new_label();

		// Condition label
		let mut condition_insts = self.compile_expression(statement.condition)?;
		condition_insts.push(Instruction::JT(JT(body_label.clone()))); // If the condition holds, jump to the body
		emitted.push(Instruction::LABEL(LABEL(
			condition_label.clone(),
			condition_insts
		)));

		// Body label
		let mut body_insts = self.compile_statements(statement.statements)?;
		body_insts.push(Instruction::JMP(JMP(condition_label.clone()))); // Jump to the condition after each iteration
		emitted.push(Instruction::LABEL(LABEL(body_label, body_insts)));

		emitted.push(Instruction::JMP(JMP(condition_label))); // After all labels have been defined, jump to the condition
		Ok(emitted)
	}

	fn compile_if(&mut self, statement: ast::If) -> Result<Vec<Instruction>> {
		let mut emitted = vec![];

		// Body label
		let body_label = self.new_label();
		let body_insts = self.compile_statements(statement.statements)?;
		emitted.push(Instruction::LABEL(LABEL(body_label.clone(), body_insts)));

		// Condition label
		let condition_label = self.new_label();
		let mut condition_insts = self.compile_expression(statement.condition)?;

		let (has_else_branch, has_else_if_branches) = (
			statement.else_branch.is_some(),
			!statement.elseif_branches.is_empty()
		);

		let mut elseif_branches = vec![];

		for (idx, branch) in statement.elseif_branches.into_iter().enumerate() {
			let branch_cond_label = self.new_label();
			let branch_body_label = self.new_label();

			let branch_cond_insts = self.compile_expression(branch.condition)?;
			let branch_body_insts = self.compile_statements(branch.statements)?;

			elseif_branches.push((
				LABEL(branch_cond_label.clone(), branch_cond_insts),
				LABEL(branch_body_label, branch_body_insts)
			));

			// Link the previous branch via a jump
			if idx < 1 {
				continue;
			}

			let prev_branch = elseif_branches.get_mut(idx - 1).unwrap();
			prev_branch.0 .1.push(Instruction::JTF(JTF(
				prev_branch.1 .0.clone(),
				branch_cond_label
			)));
		}

		let mut else_branch = None;
		if let Some(branch) = statement.else_branch.clone() {
			let branch_body_label = self.new_label();
			let branch_body_insts = self.compile_statements(branch.statements)?;

			else_branch = Some(LABEL(branch_body_label, branch_body_insts));
		}

		match (has_else_branch, has_else_if_branches) {
			(false, false) => {
				condition_insts.push(Instruction::JT(JT(body_label)));
			}

			(true, false) => {
				let else_branch = else_branch.unwrap();
				let else_label = else_branch.0.clone();

				// Inserting the `else` branch label
				emitted.push(Instruction::LABEL(else_branch));

				condition_insts.push(Instruction::JTF(JTF(body_label, else_label)));
			}

			(false, true) => {
				let first_branch_cond_label = elseif_branches
					.first()
					.map(|(cond, _)| &cond.0)
					.cloned()
					.unwrap();

				condition_insts.push(Instruction::JTF(JTF(body_label, first_branch_cond_label)));
			}

			(true, true) => {
				let else_branch = else_branch.unwrap();
				let else_label = else_branch.0.clone();

				let first_branch_cond_label = elseif_branches
					.first()
					.map(|(cond, _)| &cond.0)
					.cloned()
					.unwrap();

				let last_branch = elseif_branches.last_mut().unwrap();

				// Inserting the `else` branch label
				emitted.push(Instruction::LABEL(else_branch));

				// 1. Jump to the `if` body if the condition is true
				// 2. Otherwise, jump to the first `else-if` branch
				condition_insts.push(Instruction::JTF(JTF(body_label, first_branch_cond_label)));
				// 3. If none matches, finally jump to the `else` branch
				last_branch
					.0
					 .1
					.push(Instruction::JTF(JTF(last_branch.1 .0.clone(), else_label)));
			}
		}

		// Inserting the `else-if` branch labels
		let elseif_branches = elseif_branches
			.into_iter()
			.flat_map(|(cond, body)| vec![cond, body])
			.map(Instruction::LABEL)
			.collect::<Vec<_>>();
		emitted.extend(elseif_branches);

		// Inserting the condition label
		emitted.push(Instruction::LABEL(LABEL(
			condition_label.clone(),
			condition_insts
		)));

		emitted.push(Instruction::JMP(JMP(condition_label))); // After all labels have been defined, jump to the condition
		Ok(emitted)
	}
}

impl Default for Compiler {
	fn default() -> Self { Self::new() }
}
