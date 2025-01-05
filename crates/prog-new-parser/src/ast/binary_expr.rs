use anyhow::{bail, Result};
use prog_lexer::TokenKind;

use super::op_to_token;
use crate::ast::*;
use crate::{ASTNode, Parse, ParseStream, Span};

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

impl ASTNode<'_> for BinaryOp<'_> {
	fn span(&self) -> Span { self.span }
}

impl<'inp> Parse<'inp> for BinaryOp<'inp> {
	fn parse(input: &ParseStream<'inp>) -> Result<Self> {
		let token = input.expect_next()?;
		let span = token.span();

		match token.kind() {
			TokenKind::Plus => {
				Ok(Self {
					kind: BinaryOpKind::Plus,
					span
				})
			}

			TokenKind::Minus => {
				Ok(Self {
					kind: BinaryOpKind::Minus,
					span
				})
			}

			TokenKind::Slash => {
				Ok(Self {
					kind: BinaryOpKind::Slash,
					span
				})
			}

			TokenKind::Asterisk => {
				Ok(Self {
					kind: BinaryOpKind::Asterisk,
					span
				})
			}

			TokenKind::Sign => {
				Ok(Self {
					kind: BinaryOpKind::Sign,
					span
				})
			}

			TokenKind::EqEq => {
				Ok(Self {
					kind: BinaryOpKind::EqEq,
					span
				})
			}

			TokenKind::Neq => {
				Ok(Self {
					kind: BinaryOpKind::Neq,
					span
				})
			}

			TokenKind::And => {
				Ok(Self {
					kind: BinaryOpKind::And,
					span
				})
			}

			TokenKind::Or => {
				Ok(Self {
					kind: BinaryOpKind::Or,
					span
				})
			}

			TokenKind::Gt => {
				Ok(Self {
					kind: BinaryOpKind::Gt,
					span
				})
			}

			TokenKind::Lt => {
				Ok(Self {
					kind: BinaryOpKind::Lt,
					span
				})
			}

			TokenKind::Gte => {
				Ok(Self {
					kind: BinaryOpKind::Gte,
					span
				})
			}

			TokenKind::Lte => {
				Ok(Self {
					kind: BinaryOpKind::Lte,
					span
				})
			}

			TokenKind::LeftBracket => {
				Ok(Self {
					kind: BinaryOpKind::LeftBracket,
					span
				})
			}

			TokenKind::Dot => {
				Ok(Self {
					kind: BinaryOpKind::Dot,
					span
				})
			}

			// TODO: proper error reporting
			kind => bail!("Unknown binary operator `{token}` of type `{kind:?}`")
		}
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
