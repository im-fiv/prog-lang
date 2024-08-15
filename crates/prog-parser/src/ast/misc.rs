use super::{Expression, Position, Statement};

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Program {
	pub statements: Vec<Statement>
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct ConditionBranch {
	pub condition: Expression,
	pub statements: Vec<Statement>,
	pub position: Position
}
