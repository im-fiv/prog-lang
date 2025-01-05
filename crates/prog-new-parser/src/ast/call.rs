use anyhow::Result;

use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseStream, Position, Span, Token};

#[derive(Debug, Clone, PartialEq)]
pub struct Call<'inp> {
	pub func: Box<Term<'inp>>,
	pub _lp: token::LeftParen<'inp>,
	pub args: Box<Punctuated<'inp, Expr<'inp>, token::Comma<'inp>>>,
	pub _rp: token::RightParen<'inp>
}

impl<'inp> Call<'inp> {
	pub fn parse_with_func(input: &ParseStream<'inp>, func: Box<Term<'inp>>) -> Result<Self> {
		let _lp = input.parse::<token::LeftParen>()?;
		let args = Box::new(input.parse::<Punctuated<'inp, Expr, token::Comma>>()?);
		let _rp = input.parse::<token::RightParen>()?;

		Ok(Self {
			func,
			_lp,
			args,
			_rp
		})
	}
}

impl ASTNode for Call<'_> {
	fn span(&self) -> Span {
		let start = self.func.span().start();
		let end = Token::span(&self._rp).end();

		let source = self.func.span().source();
		let position = Position::new(start, end);

		Span::new(source, position)
	}
}

impl<'inp> Parse<'inp> for Call<'inp> {
	fn parse(input: &ParseStream<'inp>) -> Result<Self> {
		let func = Box::new(input.parse::<Term>()?);
		Self::parse_with_func(input, func)
	}
}
