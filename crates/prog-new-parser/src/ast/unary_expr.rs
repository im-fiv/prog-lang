use anyhow::{bail, Result};
use prog_lexer::TokenKind;

use super::op_to_token;
use crate::ast::*;
use crate::{ASTNode, Parse, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr<'inp> {
	pub op: UnaryOp<'inp>,
	pub operand: Term<'inp>
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnaryOp<'inp> {
	pub kind: UnaryOpKind,
	pub span: Span<'inp>
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOpKind {
	Minus,
	Not
}

impl<'inp> ASTNode<'inp> for UnaryExpr<'inp> {
	fn span(&'inp self) -> Span<'inp> {
		let start = self.op.span().start();
		let end = self.operand.span().end();

		let source = self.op.span().source();
		let position = Position::new(start, end);

		Span::new(source, position)
	}
}

impl<'inp> ASTNode<'inp> for UnaryOp<'inp> {
	fn span(&'inp self) -> Span<'inp> { self.span }
}

impl<'inp> Parse<'inp> for UnaryOp<'inp> {
	fn parse(input: &'_ ParseStream<'inp>) -> Result<Self> {
		let token = input.expect_next()?;
		let span = token.span();

		match token.kind() {
			TokenKind::Minus => {
				Ok(Self {
					kind: UnaryOpKind::Minus,
					span
				})
			}

			TokenKind::Not => {
				Ok(Self {
					kind: UnaryOpKind::Not,
					span
				})
			}

			// TODO: proper error reporting
			kind => bail!("Unknown unary operator `{token}` of type `{kind:?}`")
		}
	}
}

op_to_token!(UnaryOp : UnaryOpKind => Minus);
op_to_token!(UnaryOp : UnaryOpKind => Not);
