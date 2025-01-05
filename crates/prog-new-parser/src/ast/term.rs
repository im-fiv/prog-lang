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

impl ASTNode<'_> for Term<'_> {
	fn span(&self) -> Span {
		// TODO
		todo!()
	}
}

impl<'inp> Parse<'inp> for Term<'inp> {
	fn parse(input: &'_ ParseStream<'inp>) -> Result<Self> {
		let token = input.expect_peek()?;

		let term = match token.kind() {
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

		if input.peek_matches(TokenKind::LeftParen).is_some() {
			let func = Box::new(term.clone());
			let call_result = input.try_parse_with(|i| Call::parse_without_func(i, func));

			if let Ok(c) = call_result.map(Self::Call) {
				return Ok(c);
			}
		}

		Ok(term)
	}
}
