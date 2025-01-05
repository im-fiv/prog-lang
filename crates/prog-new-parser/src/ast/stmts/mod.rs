mod var_def;
use anyhow::Result;
use prog_lexer::TokenKind;
pub use var_def::VariableDefinition;

use crate::{ast, ASTNode, Parse, ParseStream, Position, Span};

#[derive(Debug)]
pub struct Program<'inp> {
	pub statements: Vec<Statement<'inp>>
}

#[derive(Debug)]
pub enum Statement<'inp> {
	VariableDefinition(VariableDefinition<'inp>),
	FunctionCall(ast::Call<'inp>)
}

impl ASTNode for Program<'_> {
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

impl<'inp> Parse<'inp> for Program<'inp> {
	fn parse(input: &ParseStream<'inp>) -> Result<Self> {
		let mut stmts = vec![];

		while let Some(token) = input.peek() {
			if token.kind() == TokenKind::Eof {
				break;
			}

			let stmt = input.parse::<Statement>()?;
			stmts.push(stmt);
		}

		Ok(Self { statements: stmts })
	}
}

impl ASTNode for Statement<'_> {
	fn span(&self) -> Span {
		match self {
			Self::VariableDefinition(stmt) => stmt as &dyn ASTNode,
			Self::FunctionCall(stmt) => stmt as &dyn ASTNode
		}
		.span()
	}
}

impl<'inp> Parse<'inp> for Statement<'inp> {
	fn parse(input: &ParseStream<'inp>) -> Result<Self> {
		if input.peek_matches(TokenKind::Def).is_some() {
			return input
				.parse::<VariableDefinition>()
				.map(Self::VariableDefinition);
		}

		input.parse::<ast::Call>().map(Self::FunctionCall)
	}
}
