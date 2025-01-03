use anyhow::Result;

use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseStream, Span};

#[derive(Debug)]
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
	fn parse(input: &'inp ParseStream<'inp>) -> Result<Self> {
		use prog_lexer::TokenKind;

		// TODO: error handling
		let token = input.peek().unwrap();

		Ok(match token.kind() {
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
		})
	}
}
