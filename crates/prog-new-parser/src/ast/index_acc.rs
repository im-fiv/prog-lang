use anyhow::Result;

use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct IndexAcc<'inp> {
	pub list: Box<Term<'inp>>,
	pub _lb: token::LeftBracket<'inp>,
	pub index: Box<Expr<'inp>>,
	pub _rb: token::RightBracket<'inp>
}

impl<'inp> ASTNode<'inp> for IndexAcc<'inp> {
	fn span(&'inp self) -> Span<'inp> {
		let start = self.list.span().start();
		let end = self._rb.span().end();

		let source = self.list.span().source();
		let position = Position::new(start, end);

		Span::new(source, position)
	}
}

impl<'inp> Parse<'inp> for IndexAcc<'inp> {
	fn parse(input: &'_ ParseStream<'inp>) -> Result<Self> {
		let list = Box::new(input.parse::<Term>()?);
		let _lb = input.parse::<token::LeftBracket>()?;
		let index = Box::new(input.parse::<Expr>()?);
		let _rb = input.parse::<token::RightBracket>()?;

		Ok(Self {
			list,
			_lb,
			index,
			_rb
		})
	}
}
