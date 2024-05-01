use super::expressions::{Expression, Call};
use super::misc::ConditionBranch;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
	VariableDefine {
		name: String,
		value: Option<Expression>
	},

	VariableAssign {
		name: String,
		value: Expression
	},

	DoBlock(Vec<Statement>),
	Return(Option<Expression>),
	Call(Call),

	WhileLoop {
		condition: Expression,
		statements: Vec<Statement>
	},

	Break,
	Continue,

	If {
		condition: Expression,
		statements: Vec<Statement>,
		elseif_branches: Vec<ConditionBranch>,
		else_branch: Option<ConditionBranch>
	},

	ExpressionAssign {
		expression: Expression,
		value: Expression
	}
}