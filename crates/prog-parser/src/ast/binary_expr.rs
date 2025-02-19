use std::fmt::{self, Display};

use prog_lexer::TokenKind;

use super::op_to_token;
use crate::ast::*;
use crate::{
	error, ASTNode, Parse, ParseError, ParseErrorKind, ParseResult, ParseStream, Position, Span
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct BinaryExpr<'src> {
	pub lhs: Term<'src>,
	pub op: BinaryOp<'src>,
	pub rhs: Term<'src>
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct BinaryOp<'src> {
	pub kind: BinaryOpKind,
	pub span: Span<'src>
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum BinaryOpKind {
	Plus,
	Minus,
	Asterisk,
	Slash,
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

impl<'src> ASTNode<'src> for BinaryExpr<'src> {
	fn span<'a>(&'a self) -> Span<'src> {
		let start = self.lhs.start();
		let end = self.rhs.end();

		let source = self.lhs.source();
		let file = self.lhs.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'src> ASTNode<'src> for BinaryOp<'src> {
	fn span<'a>(&'a self) -> Span<'src> { self.span }
}

impl<'src> Parse<'src> for BinaryOp<'src> {
	fn parse(input: &ParseStream<'src, '_>) -> ParseResult<'src, Self> {
		let token = input.expect_next()?;
		Self::try_from(&token as &dyn crate::Token)
	}
}

impl<'src> TryFrom<&dyn crate::Token<'src>> for BinaryOp<'src> {
	type Error = ParseError<'src>;

	fn try_from(token: &dyn crate::Token<'src>) -> std::result::Result<Self, Self::Error> {
		let span = token.sp();
		let kind = BinaryOpKind::try_from(token.tk())
			.map_err(|e| ParseError::new(span, ParseErrorKind::Internal(error::Internal(e))))?;

		Ok(Self { kind, span })
	}
}

impl TryFrom<TokenKind> for BinaryOpKind {
	type Error = String;

	fn try_from(kind: TokenKind) -> std::result::Result<Self, Self::Error> {
		use TokenKind as T;

		Ok(match kind {
			T::Plus => Self::Plus,
			T::Minus => Self::Minus,
			T::Asterisk => Self::Asterisk,
			T::Slash => Self::Slash,
			T::Sign => Self::Sign,
			T::EqEq => Self::EqEq,
			T::Neq => Self::Neq,
			T::And => Self::And,
			T::Or => Self::Or,
			T::Gt => Self::Gt,
			T::Lt => Self::Lt,
			T::Gte => Self::Gte,
			T::Lte => Self::Lte,
			T::LeftBracket => Self::LeftBracket,
			T::Dot => Self::Dot,

			kind => return Err(format!("Unknown binary operator of type `{kind:?}`"))
		})
	}
}

impl From<BinaryOpKind> for TokenKind {
	fn from(kind: BinaryOpKind) -> Self {
		use BinaryOpKind as B;

		match kind {
			B::Plus => Self::Plus,
			B::Minus => Self::Minus,
			B::Asterisk => Self::Asterisk,
			B::Slash => Self::Slash,
			B::Sign => Self::Sign,
			B::EqEq => Self::EqEq,
			B::Neq => Self::Neq,
			B::And => Self::And,
			B::Or => Self::Or,
			B::Gt => Self::Gt,
			B::Lt => Self::Lt,
			B::Gte => Self::Gte,
			B::Lte => Self::Lte,
			B::LeftBracket => Self::LeftBracket,
			B::Dot => Self::Dot
		}
	}
}

// Formatting is directly delegated to `TokenKind`
impl Display for BinaryOpKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { TokenKind::from(*self).fmt(f) }
}

op_to_token!(BinaryOp : BinaryOpKind => Plus);
op_to_token!(BinaryOp : BinaryOpKind => Minus);
op_to_token!(BinaryOp : BinaryOpKind => Asterisk);
op_to_token!(BinaryOp : BinaryOpKind => Slash);
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
