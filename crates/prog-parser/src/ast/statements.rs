use super::Position;
use super::expressions::{Expression, Call};
use super::misc::ConditionBranch;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
	VariableDefine {
		name: (String, Position),
		value: Option<Expression>,
		position: Position
	},

	VariableAssign {
		name: (String, Position),
		value: Expression,
		position: Position
	},

	DoBlock(Vec<Statement>, Position),
	Return(Option<Expression>, Position),
	Call(Call),

	WhileLoop {
		condition: Expression,
		statements: Vec<Statement>,
		position: Position
	},

	Break(Position),
	Continue(Position),

	If {
		condition: Expression,
		statements: Vec<Statement>,
		elseif_branches: Vec<ConditionBranch>,
		else_branch: Option<ConditionBranch>,
		position: Position
	},

	ExpressionAssign {
		expression: Expression,
		value: Expression,
		position: Position
	}
}

impl Statement {
	pub fn position(&self) -> Position {
		match self {
			Self::VariableDefine { name: _, value: _, position } => position,
			Self::VariableAssign { name: _, value: _, position } => position,
			Self::DoBlock(_, position) => position,
			Self::Return(_, position) => position,
			Self::Call(call) => &call.position,
			Self::WhileLoop { condition: _, statements: _, position } => position,
			Self::Break(position) => position,
			Self::Continue(position) => position,
			Self::If { condition: _, statements: _, elseif_branches: _, else_branch: _, position } => position,
			Self::ExpressionAssign { expression: _, value: _, position } => position
		}.to_owned()
	}
}