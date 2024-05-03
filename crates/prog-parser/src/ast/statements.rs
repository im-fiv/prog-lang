use super::Position;
use super::expressions::{Expression, Call};
use super::misc::ConditionBranch;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
	VariableDefine {
		name: String,
		value: Option<Expression>,
		position: Position
	},

	VariableAssign {
		name: String,
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