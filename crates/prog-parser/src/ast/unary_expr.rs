use prog_lexer::TokenKind;

use crate::ast::*;
use crate::{
	errors, ASTNode, Parse, ParseError, ParseErrorKind, ParseResult, ParseStream, Position, Span
};

use super::op_to_token;

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

impl<'inp> Parse<'inp> for UnaryOp<'inp> {
	fn parse(input: &ParseStream<'inp>) -> ParseResult<Self> {
		let token = input.expect_next()?;
		Self::try_from(&token as &dyn crate::Token)
	}
}

impl<'inp> TryFrom<&dyn crate::Token<'inp>> for UnaryOp<'inp> {
	type Error = ParseError;

	fn try_from(token: &dyn crate::Token<'inp>) -> std::result::Result<Self, Self::Error> {
		let span = token.sp();
		let kind = UnaryOpKind::try_from(token.tk()).map_err(|e| {
			ParseError::new(
				span.source().to_owned(),
				span.file().to_owned(),
				span.position(),
				ParseErrorKind::Internal(errors::Internal(e))
			)
		})?;

		Ok(Self { kind, span })
	}
}

impl TryFrom<TokenKind> for UnaryOpKind {
	type Error = String;

	fn try_from(kind: TokenKind) -> std::result::Result<Self, Self::Error> {
		use {TokenKind as T, UnaryOpKind as U};

		Ok(match kind {
			T::Minus => U::Minus,
			T::Not => U::Not,

			kind => return Err(format!("Unknown unary operator of type `{kind:?}`"))
		})
	}
}

op_to_token!(UnaryOp : UnaryOpKind => Minus);
op_to_token!(UnaryOp : UnaryOpKind => Not);
