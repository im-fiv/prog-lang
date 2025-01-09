use std::fmt::{self, Display};

use prog_lexer::TokenKind;

use crate::ast::*;
use crate::{
	error, ASTNode, Parse, ParseError, ParseErrorKind, ParseResult, ParseStream, Position, Span
};

use super::op_to_token;

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr<'src> {
	pub op: UnaryOp<'src>,
	pub operand: Term<'src>
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnaryOp<'src> {
	pub kind: UnaryOpKind,
	pub span: Span<'src>
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOpKind {
	Minus,
	Not
}

impl ASTNode for UnaryExpr<'_> {
	fn span(&self) -> Span {
		let start = self.op.start();
		let end = self.operand.end();

		let source = self.op.source();
		let file = self.op.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl ASTNode for UnaryOp<'_> {
	fn span(&self) -> Span { self.span }
}

impl<'src> Parse<'src> for UnaryOp<'src> {
	fn parse(input: &ParseStream<'src>) -> ParseResult<Self> {
		let token = input.expect_next()?;
		Self::try_from(&token as &dyn crate::Token)
	}
}

impl<'src> TryFrom<&dyn crate::Token<'src>> for UnaryOp<'src> {
	type Error = ParseError;

	fn try_from(token: &dyn crate::Token<'src>) -> std::result::Result<Self, Self::Error> {
		let span = token.sp();
		let kind = UnaryOpKind::try_from(token.tk()).map_err(|e| {
			ParseError::with_span(span, ParseErrorKind::Internal(error::Internal(e)))
		})?;

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

impl Display for UnaryOpKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Minus => write!(f, "-"),
			Self::Not => write!(f, "not")
		}
	}
}

op_to_token!(UnaryOp : UnaryOpKind => Minus);
op_to_token!(UnaryOp : UnaryOpKind => Not);
