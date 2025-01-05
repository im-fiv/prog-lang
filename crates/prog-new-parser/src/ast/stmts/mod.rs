mod var_def;
pub use var_def::VariableDefinition;

use crate::{ast, ASTNode, Position, Span};

#[derive(Debug)]
pub struct Program<'inp> {
	pub statements: Vec<Statement<'inp>>
}

#[derive(Debug)]
pub enum Statement<'inp> {
	VariableDefinition(VariableDefinition<'inp>),
	FunctionCall(ast::Call<'inp>)
}

impl ASTNode<'_> for Program<'_> {
	fn span(&self) -> Span {
		assert!(
			!self.statements.is_empty(),
			"Could not get program's span as it is empty"
		);

		let first = self.statements.first().unwrap().span();
		let last = self.statements.last().unwrap().span();

		let start = first.start();
		let end = last.end();

		let source = first.source();
		let position = Position::new(start, end);

		Span::new(source, position)
	}
}

impl ASTNode<'_> for Statement<'_> {
	fn span(&self) -> Span {
		match self {
			Self::VariableDefinition(stmt) => stmt as &dyn ASTNode,
			Self::FunctionCall(stmt) => stmt as &dyn ASTNode
		}
		.span()
	}
}
