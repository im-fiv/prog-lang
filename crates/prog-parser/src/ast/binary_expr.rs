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

impl ASTNode for BinaryExpr<'_> {
	fn span(&self) -> Span {
		let start = self.lhs.start();
		let end = self.rhs.end();

		let source = self.lhs.source();
		let position = Position::new(start, end);

		Span::new(source, position)
	}
}

impl ASTNode for BinaryOp<'_> {
	fn span(&self) -> Span { self.span }
}

impl<'inp> Parse<'inp> for BinaryOp<'inp> {
	fn parse(input: &ParseStream<'inp>) -> Result<Self> {
		let token = input.expect_next()?;

		let span = token.span();
		let kind = BinaryOpKind::try_from(token.kind())?;

		Ok(Self { kind, span })
	}
}

impl<'inp> TryFrom<&'inp dyn crate::Token> for BinaryOp<'inp> {
	type Error = anyhow::Error;

	fn try_from(token: &'inp dyn crate::Token) -> std::result::Result<Self, Self::Error> {
		let span = token.sp();
		let kind = BinaryOpKind::try_from(token.tk())?;

		Ok(Self { kind, span })
	}
}

impl TryFrom<TokenKind> for BinaryOpKind {
	type Error = anyhow::Error;

	fn try_from(kind: TokenKind) -> std::result::Result<Self, Self::Error> {
		use {TokenKind as T, BinaryOpKind as B};

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
