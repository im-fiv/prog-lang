use prog_lexer::TokenKind;

use super::op_to_token;
use crate::ast::*;
use crate::{
	error, ASTNode, Parse, ParseError, ParseErrorKind, ParseResult, ParseStream, Position, Span
};

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr<'src> {
	pub lhs: Term<'src>,
	pub op: BinaryOp<'src>,
	pub rhs: Term<'src>
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BinaryOp<'src> {
	pub kind: BinaryOpKind,
	pub span: Span<'src>
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
		let file = self.lhs.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl ASTNode for BinaryOp<'_> {
	fn span(&self) -> Span { self.span }
}

impl<'src> Parse<'src> for BinaryOp<'src> {
	fn parse(input: &ParseStream<'src>) -> ParseResult<Self> {
		let token = input.expect_next()?;
		Self::try_from(&token as &dyn crate::Token)
	}
}

impl<'src> TryFrom<&dyn crate::Token<'src>> for BinaryOp<'src> {
	type Error = ParseError;

	fn try_from(token: &dyn crate::Token<'src>) -> std::result::Result<Self, Self::Error> {
		let span = token.sp();
		let kind = BinaryOpKind::try_from(token.tk()).map_err(|e| {
			ParseError::new(
				span.source().to_owned(),
				span.file().to_owned(),
				span.position(),
				ParseErrorKind::Internal(error::Internal(e))
			)
		})?;

		Ok(Self { kind, span })
	}
}

impl TryFrom<TokenKind> for BinaryOpKind {
	type Error = String;

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

			kind => return Err(format!("Unknown binary operator of type `{kind:?}`"))
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
