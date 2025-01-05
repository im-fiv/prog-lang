use anyhow::Result;

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
	fn parse(input: &ParseStream<'inp>) -> Result<Self> {
		use prog_lexer::TokenKind;

		// TODO: error handling
		let token = input.peek().unwrap();

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
			let fork = input.fork();

			let func = Box::new(term.clone());
			let call_result = Call::parse_without_func(&fork, func);

			if let Ok(c) = call_result.map(Self::Call) {
				input.set_cursor(fork.cursor());
				return Ok(c);
			}
		}

		Ok(term)
	}
}
