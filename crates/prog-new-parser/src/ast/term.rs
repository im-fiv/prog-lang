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

impl ASTNode for Term<'_> {
	fn span(&self) -> Span {
		// TODO
		todo!()
	}
}

impl<'inp> Parse<'inp> for Term<'inp> {
	fn parse(input: &'_ ParseStream<'inp>) -> Result<Self> {
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
