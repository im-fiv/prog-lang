use anyhow::Result;

use crate::{token, ASTNode, Parse, ParseStream, Span, Position};
use crate::ast::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Extern<'inp> {
	pub _extern: token::Extern<'inp>,
	pub value: Box<Expr<'inp>>
}

impl ASTNode for Extern<'_> {
	fn span(&self) -> Span {
		let start = self._extern.start();
		let end = self.value.end();

		let source = self._extern.source();
		let position = Position::new(start, end);

		Span::new(source, position)
	}
}

impl<'inp> Parse<'inp> for Extern<'inp> {
	fn parse(input: &ParseStream<'inp>) -> Result<Self> {
		let _extern = input.parse::<token::Extern>()?;
		let value = Box::new(input.parse::<Expr>()?);

		Ok(Self {
			_extern,
			value
		})
	}
}
