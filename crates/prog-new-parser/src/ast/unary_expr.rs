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

impl ASTNode for UnaryExpr<'_> {
	fn span(&self) -> Span {
		let start = self.op.span().start();
		let end = self.operand.span().end();

		let source = self.op.span().source();
		let position = Position::new(start, end);

		Span::new(source, position)
	}
}

impl ASTNode for UnaryOp<'_> {
	fn span(&self) -> Span { self.span }
}

impl<'inp> Parse<'inp> for UnaryOp<'inp> {
	fn parse(input: &'_ ParseStream<'inp>) -> Result<Self> {
		let token = input.expect_next()?;

		let kind = UnaryOpKind::try_from(token.kind())?;
		let span = token.span();

		Ok(Self { kind, span })
	}
}

impl TryFrom<TokenKind> for UnaryOpKind {
	type Error = anyhow::Error;

	fn try_from(kind: TokenKind) -> std::result::Result<Self, Self::Error> {
		use {TokenKind as T, UnaryOpKind as U};

		Ok(match kind {
			T::Minus => U::Minus,
			T::Not => U::Not,

			// TODO: proper error reporting
			kind => bail!("Unknown unary operator of type `{kind:?}`")
		})
	}
}

op_to_token!(UnaryOp : UnaryOpKind => Minus);
op_to_token!(UnaryOp : UnaryOpKind => Not);
