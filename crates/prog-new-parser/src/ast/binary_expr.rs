use anyhow::{bail, Result};
use prog_lexer::TokenKind;

use super::op_to_token;
use crate::ast::*;
use crate::{ASTNode, Parse, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr<'inp> {
	pub lhs: Term<'inp>,
	pub op: BinaryOp<'inp>,
	pub rhs: Term<'inp>
}

impl ASTNode for BinaryExpr<'_> {
	fn span(&self) -> Span {
		let start = self.lhs.span().start();
		let end = self.rhs.span().end();

		let source = self.lhs.span().source();
		let position = Position::new(start, end);

		Span::new(source, position)
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BinaryOp<'inp> {
	pub kind: BinaryOpKind,
	pub span: Span<'inp>
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOpKind {
	Plus,
	Minus,
	Slash,
	Asterisk,
	Sign,
	EqEq,
	Neq,
	And,
	Or,
	Gt,
	Lt,
	Gte,
	Lte,
	LeftBracket,
	Dot
}

impl ASTNode for BinaryOp<'_> {
	fn span(&self) -> Span { self.span }
}

impl<'inp> Parse<'inp> for BinaryOp<'inp> {
	fn parse(input: &'_ ParseStream<'inp>) -> Result<Self> {
		let token = input.expect_next()?;

		let span = token.span();
		let kind = BinaryOpKind::try_from(token.kind())?;

		Ok(Self { kind, span })
	}
}

impl TryFrom<TokenKind> for BinaryOpKind {
	type Error = anyhow::Error;

	fn try_from(kind: TokenKind) -> std::result::Result<Self, Self::Error> {
		use {BinaryOpKind as B, TokenKind as T};

		Ok(match kind {
			T::Plus => B::Plus,
			T::Minus => B::Minus,
			T::Slash => B::Slash,
			T::Asterisk => B::Asterisk,
			T::Sign => B::Sign,
			T::EqEq => B::EqEq,
			T::Neq => B::Neq,
			T::And => B::And,
			T::Or => B::Or,
			T::Gt => B::Gt,
			T::Lt => B::Lt,
			T::Gte => B::Gte,
			T::Lte => B::Lte,
			T::LeftBracket => B::LeftBracket,
			T::Dot => B::Dot,

			// TODO: proper error reporting
			kind => bail!("Unknown binary operator of type `{kind:?}`")
		})
	}
}

op_to_token!(BinaryOp : BinaryOpKind => Plus);
op_to_token!(BinaryOp : BinaryOpKind => Minus);
op_to_token!(BinaryOp : BinaryOpKind => Slash);
op_to_token!(BinaryOp : BinaryOpKind => Asterisk);
op_to_token!(BinaryOp : BinaryOpKind => Sign);
op_to_token!(BinaryOp : BinaryOpKind => EqEq);
op_to_token!(BinaryOp : BinaryOpKind => Neq);
op_to_token!(BinaryOp : BinaryOpKind => And);
op_to_token!(BinaryOp : BinaryOpKind => Or);
op_to_token!(BinaryOp : BinaryOpKind => Gt);
op_to_token!(BinaryOp : BinaryOpKind => Lt);
op_to_token!(BinaryOp : BinaryOpKind => Gte);
op_to_token!(BinaryOp : BinaryOpKind => Lte);
op_to_token!(BinaryOp : BinaryOpKind => LeftBracket);
op_to_token!(BinaryOp : BinaryOpKind => Dot);
