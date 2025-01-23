use std::fmt::{self, Display};

use prog_lexer::TokenKind;

use crate::ast::*;
use crate::{
	error, ASTNode, Parse, ParseError, ParseErrorKind, ParseResult, ParseStream, Position, Span
};

use super::op_to_token;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct UnaryExpr<'src> {
	pub op: UnaryOp<'src>,
	pub operand: Term<'src>
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct UnaryOp<'src> {
	pub kind: UnaryOpKind,
	pub span: Span<'src>
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum UnaryOpKind {
	Minus,
	Not
}

impl<'src> ASTNode<'src> for UnaryExpr<'src> {
	fn span<'a>(&'a self) -> Span<'src> {
		let start = self.op.start();
		let end = self.operand.end();

		let source = self.op.source();
		let file = self.op.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'src> ASTNode<'src> for UnaryOp<'src> {
	fn span<'a>(&'a self) -> Span<'src> { self.span }
}

impl<'src> Parse<'src> for UnaryExpr<'src> {
	fn parse(input: &ParseStream<'src, '_>) -> ParseResult<'src, Self> {
		let op = input.parse::<UnaryOp>()?;
		let operand = input.parse::<Term>()?;

		Ok(Self { op, operand })
	}
}

impl<'src> Parse<'src> for UnaryOp<'src> {
	fn parse(input: &ParseStream<'src, '_>) -> ParseResult<'src, Self> {
		let token = input.expect_next()?;
		Self::try_from(&token as &dyn crate::Token)
	}
}

impl<'src> TryFrom<&dyn crate::Token<'src>> for UnaryOp<'src> {
	type Error = ParseError<'src>;

	fn try_from(token: &dyn crate::Token<'src>) -> std::result::Result<Self, Self::Error> {
		let span = token.sp();
		let kind = UnaryOpKind::try_from(token.tk())
			.map_err(|e| ParseError::new(span, ParseErrorKind::Internal(error::Internal(e))))?;

		Ok(Self { kind, span })
	}
}

impl TryFrom<TokenKind> for UnaryOpKind {
	type Error = String;

	fn try_from(kind: TokenKind) -> std::result::Result<Self, Self::Error> {
		use TokenKind as T;

		Ok(match kind {
			T::Minus => Self::Minus,
			T::Not => Self::Not,

			kind => return Err(format!("Unknown unary operator of type `{kind:?}`"))
		})
	}
}

impl From<UnaryOpKind> for TokenKind {
	fn from(kind: UnaryOpKind) -> Self {
		use UnaryOpKind as U;

		match kind {
			U::Minus => Self::Minus,
			U::Not => Self::Not
		}
	}
}

// Formatting is directly delegated to `TokenKind`
impl Display for UnaryOpKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { TokenKind::from(*self).fmt(f) }
}

op_to_token!(UnaryOp : UnaryOpKind => Minus);
op_to_token!(UnaryOp : UnaryOpKind => Not);
