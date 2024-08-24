mod intermediate;

use anyhow::Result;
use prog_parser::ast;
use prog_vm::instruction::*;
use prog_vm::Value;

#[derive(Debug)]
pub struct Compiler {
	label_counter: usize
}

impl Compiler {
	pub fn new() -> Self { Self { label_counter: 0 } }

	pub fn compile(&mut self, ast: ast::Program) -> Result<Bytecode> {
		Ok(Bytecode::new(self.compile_statements(ast.statements)?))
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
		use intermediate::{ConditionalBranch, IntermediateLabel};

		let mut emitted = vec![];

		let mut r#while = ConditionalBranch {
			condition: IntermediateLabel {
				name: self.new_label(),
				instructions: self.compile_expression(statement.condition)?
			},

			body: IntermediateLabel {
				name: self.new_label(),
				instructions: self.compile_statements(statement.statements)?
			}
		};

		// If the condition holds, jump to the body
		r#while
			.condition
			.instructions
			.push(Instruction::JT(JT(r#while.body.name.clone())));

		// Jump to the condition after each iteration
		r#while
			.body
			.instructions
			.push(Instruction::JMP(JMP(r#while.condition.name.clone())));

		emitted.extend(r#while.condition.flatten());
		emitted.extend(r#while.body.flatten());

		// After all labels have been defined, jump to the condition
		emitted.push(Instruction::JMP(JMP(r#while.condition.name)));
		Ok(emitted)
	}

	// TODO: fix jumps outer->condition (falsy)->outer
	fn compile_if(&mut self, statement: ast::If) -> Result<Vec<Instruction>> {
		use intermediate::{ConditionalBranch, IntermediateLabel, UnconditionalBranch};

		let mut emitted = vec![];

		let mut r#if = ConditionalBranch {
			condition: IntermediateLabel {
				name: self.new_label(),
				instructions: self.compile_expression(statement.condition)?
			},

			body: IntermediateLabel {
				name: self.new_label(),
				instructions: self.compile_statements(statement.statements)?
			}
		};
		emitted.extend(r#if.body.flatten());

		let mut elseif_branches = vec![];

		for (idx, branch) in statement.elseif_branches.into_iter().enumerate() {
			let branch = ConditionalBranch {
				condition: IntermediateLabel {
					name: self.new_label(),
					instructions: self.compile_expression(branch.condition)?
				},

				body: IntermediateLabel {
					name: self.new_label(),
					instructions: self.compile_statements(branch.statements)?
				}
			};

			// Linking the last branch to the current via a jump
			if idx < 1 {
				continue;
			}

			let prev_branch: &mut ConditionalBranch = elseif_branches.last_mut().unwrap();
			prev_branch
				.condition
				.instructions
				.push(Instruction::JTF(JTF(
					prev_branch.body.name.clone(),
					branch.condition.name.clone()
				)));

			elseif_branches.push(branch);
		}

		let else_branch = statement
			.else_branch
			.map(|branch| -> Result<UnconditionalBranch> {
				Ok(UnconditionalBranch {
					body: IntermediateLabel {
						name: self.new_label(),
						instructions: self.compile_statements(branch.statements)?
					}
				})
			})
			.transpose()?;

		match (else_branch, !elseif_branches.is_empty()) {
			(None, false) => {
				// If no branches are present, generate a jump to the `if` body
				// if the condition is truthy
				r#if.condition
					.instructions
					.push(Instruction::JT(JT(r#if.body.name)));
			}

			(Some(else_branch), false) => {
				// Inserting the `else` branch label
				emitted.extend(else_branch.body.flatten());

				// Emitting a `JTF` instruction:
				// If the `if` condition is truthy, jump to the `if` body.
				// Otherwise, jump to the `else` branch's body
				r#if.condition
					.instructions
					.push(Instruction::JTF(JTF(r#if.body.name, else_branch.body.name)));
			}

			(None, true) => {
				let first_branch = elseif_branches.first().unwrap();

				// Emitting a `JTF` instruction:
				// If the `if` condition is truthy, jump to the `if` body.
				// Otherwise, jump to the first `else-if` branch's body
				r#if.condition.instructions.push(Instruction::JTF(JTF(
					r#if.body.name,
					first_branch.condition.name.clone()
				)));
			}

			(Some(else_branch), true) => {
				let first_branch = elseif_branches.first().cloned().unwrap();
				let last_branch = elseif_branches.last_mut().unwrap();

				// Inserting the `else` branch label
				emitted.extend(else_branch.body.flatten());

				// 1. Jump to the `if` body if the condition is true
				// 2. Otherwise, jump to the first `else-if` branch's condition
				r#if.condition.instructions.push(Instruction::JTF(JTF(
					r#if.body.name,
					first_branch.condition.name.clone()
				)));
				// 3. If not a single `else-if` branch's condition matches, finally jump to the `else` branch's body
				last_branch
					.condition
					.instructions
					.push(Instruction::JTF(JTF(
						last_branch.body.name.clone(),
						else_branch.body.name
					)));
			}
		}

		for branch in elseif_branches {
			emitted.extend(branch.condition.flatten());
			emitted.extend(branch.body.flatten());
		}

		// Inserting the condition label
		emitted.extend(r#if.condition.flatten());

		emitted.push(Instruction::JMP(JMP(r#if.condition.name))); // After all labels have been defined, jump to the condition

		Ok(emitted)
	}
}

impl Default for Compiler {
	fn default() -> Self { Self::new() }
}
