use anyhow::Result;
use prog_lexer::TokenKind;

use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseStream, Span};

#[derive(Debug, Clone, PartialEq)]
pub enum Term<'inp> {
	Expr(Box<Expr<'inp>>),

	ParenExpr(ParenExpr<'inp>),
	Lit(Lit<'inp>),
	Ident(token::Ident<'inp>),
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
			TokenKind::Number | TokenKind::True | TokenKind::False | TokenKind::String => {
				Term::Lit(input.parse::<Lit>()?)
			}

			TokenKind::Ident => {
				input.next();
				Term::Ident(token::Ident::try_from(token).unwrap())
			}

			TokenKind::LeftParen => Term::ParenExpr(input.parse::<ParenExpr>()?),

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
		// TODO
		todo!()
	}
}

impl<'inp> Parse<'inp> for Term<'inp> {
	fn parse(input: &ParseStream<'inp>) -> Result<Self> { Self::parse_bounded(input, false) }
}
