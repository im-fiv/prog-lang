use anyhow::Result;
use prog_lexer::TokenKind;

use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseStream, Span};

#[derive(Debug, Clone, PartialEq)]
pub enum Term<'inp> {
	// Wrapping terms
	Expr(Box<Expr<'inp>>),
	ParenExpr(ParenExpr<'inp>),

	// Regular terms
	Lit(Lit<'inp>),
	Ident(token::Ident<'inp>),
	Func(Func<'inp>),
	List(List<'inp>),
	Obj(Obj<'inp>),
	Extern(Extern<'inp>),

	// Lookahead terms
	Call(Call<'inp>),
	IndexAcc(IndexAcc<'inp>),
	FieldAcc(FieldAcc<'inp>)
}

impl<'inp> Term<'inp> {
	/// Unlike the `Parse` implementation, does not parse more than it has to.
	///
	/// Useful when the parse call originates from `Term`'s variants to prevent
	/// `Term` from consuming the tokens that its variant was supposed to consume.
	pub fn parse_bounded(input: &ParseStream<'inp>, bounded: bool) -> Result<Self> {
		let token = input.expect_peek()?;

		let mut term = match token.kind() {
			TokenKind::LeftParen => Self::ParenExpr(input.parse::<ParenExpr>()?),

			TokenKind::Number | TokenKind::True | TokenKind::False | TokenKind::String => {
				Self::Lit(input.parse::<Lit>()?)
			}

			TokenKind::Ident => {
				input.next();
				Self::Ident(token::Ident::try_from(token).unwrap())
			}

			TokenKind::Func => Self::Func(input.parse::<Func>()?),
			TokenKind::LeftBracket => Self::List(input.parse::<List>()?),
			TokenKind::LeftBrace => Self::Obj(input.parse::<Obj>()?),
			TokenKind::Extern => Self::Extern(input.parse::<Extern>()?),

			// TODO
			t => todo!("term `{t:?}` is not yet supported")
		};

		if bounded {
			return Ok(term);
		}

		while let Some(token) = input.peek() {
			match token.kind() {
				TokenKind::LeftParen => {
					term = Self::Call(Call::parse_with_func(input, Box::new(term))?);
				}

				TokenKind::LeftBracket => {
					term = Self::IndexAcc(IndexAcc::parse_with_list(input, Box::new(term))?);
				}

				TokenKind::Dot => {
					term = Self::FieldAcc(FieldAcc::parse_with_object(input, Box::new(term))?);
				}

				_ => break
			}
		}

		Ok(term)
	}
}

impl ASTNode for Term<'_> {
	fn span(&self) -> Span {
		match self {
			Self::Expr(t) => &**t as &dyn ASTNode,
			Self::ParenExpr(t) => t as &dyn ASTNode,
			Self::Lit(t) => t as &dyn ASTNode,
			Self::Ident(t) => t as &dyn ASTNode,
			Self::Func(t) => t as &dyn ASTNode,
			Self::List(t) => t as &dyn ASTNode,
			Self::Obj(t) => t as &dyn ASTNode,
			Self::Extern(t) => t as &dyn ASTNode,
			Self::Call(t) => t as &dyn ASTNode,
			Self::IndexAcc(t) => t as &dyn ASTNode,
			Self::FieldAcc(t) => t as &dyn ASTNode
		}
		.span()
	}
}

impl<'inp> Parse<'inp> for Term<'inp> {
	fn parse(input: &ParseStream<'inp>) -> Result<Self> { Self::parse_bounded(input, false) }
}
