use anyhow::Result;

use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct WhileLoop<'inp> {
	pub _while: token::While<'inp>,
	pub cond: Expr<'inp>,
	pub block: DoBlock<'inp>
}

impl ASTNode for WhileLoop<'_> {
	fn span(&self) -> Span {
		let start = self._while.start();
		let end = self.block.end();

		let source = self._while.source();
		let position = Position::new(start, end);

		Span::new(source, position)
	}
}

impl<'inp> Parse<'inp> for WhileLoop<'inp> {
	fn parse(input: &ParseStream<'inp>) -> Result<Self> {
		let _while = input.parse::<token::While>()?;
		let cond = input.parse::<Expr>()?;
		let block = input.parse::<DoBlock>()?;

		Ok(Self {
			_while,
			cond,
			block
		})
	}
}
