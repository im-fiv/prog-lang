use prog_utils::impl_basic_conv;

use super::expressions::{Call, Expression};
use super::misc::ConditionBranch;
use super::Position;

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Statement {
	VariableDefine(VariableDefine),
	VariableAssign(VariableAssign),
	DoBlock(DoBlock),
	Return(Return),
	Call(Call),
	WhileLoop(WhileLoop),
	Break(Break),
	Continue(Continue),
	If(If),
	ExpressionAssign(ExpressionAssign)
}

impl Statement {
	pub fn position(&self) -> &Position {
		match self {
			Self::VariableDefine(statement) => &statement.position,
			Self::VariableAssign(statement) => &statement.position,
			Self::DoBlock(statement) => &statement.position,
			Self::Return(statement) => &statement.position,
			Self::Call(statement) => &statement.position,
			Self::WhileLoop(statement) => &statement.position,
			Self::Break(statement) => &statement.position,
			Self::Continue(statement) => &statement.position,
			Self::If(statement) => &statement.position,
			Self::ExpressionAssign(statement) => &statement.position
		}
	}

	pub fn name(&self) -> String {
		let name = match self {
			Self::VariableDefine(_) => "VariableDefine",
			Self::VariableAssign(_) => "VariableAssign",
			Self::DoBlock(_) => "DoBlock",
			Self::Return(_) => "Return",
			Self::Call(_) => "Call",
			Self::WhileLoop(_) => "WhileLoop",
			Self::Break(_) => "Break",
			Self::Continue(_) => "Continue",
			Self::If(_) => "If",
			Self::ExpressionAssign(_) => "ExpressionAssign"
		};

		name.to_owned()
	}
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct VariableDefine {
	pub name: (String, Position),
	pub value: Option<Expression>,
	pub position: Position
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct VariableAssign {
	pub name: (String, Position),
	pub value: Expression,
	pub position: Position
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct DoBlock {
	pub statements: Vec<Statement>,
	pub position: Position
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Return {
	pub expression: Option<Expression>,
	pub position: Position
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct WhileLoop {
	pub condition: Expression,
	pub statements: Vec<Statement>,
	pub position: Position
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Break {
	pub position: Position
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Continue {
	pub position: Position
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct If {
	pub condition: Expression,
	pub statements: Vec<Statement>,
	pub elseif_branches: Vec<ConditionBranch>,
	pub else_branch: Option<ConditionBranch>,
	pub position: Position
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct ExpressionAssign {
	pub expression: Expression,
	pub value: Expression,
	pub position: Position
}

impl_basic_conv!(from VariableDefine => Statement as VariableDefine);
impl_basic_conv!(from VariableAssign => Statement as VariableAssign);
impl_basic_conv!(from DoBlock => Statement as DoBlock);
impl_basic_conv!(from Return => Statement as Return);
impl_basic_conv!(from Call => Statement as Call);
impl_basic_conv!(from WhileLoop => Statement as WhileLoop);
impl_basic_conv!(from Break => Statement as Break);
impl_basic_conv!(from Continue => Statement as Continue);
impl_basic_conv!(from If => Statement as If);
impl_basic_conv!(from ExpressionAssign => Statement as ExpressionAssign);
