use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseResult, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct WhileLoop<'src> {
	pub _while: token::While<'src>,
	pub cond: Expr<'src>,
	pub block: DoBlock<'src>
}

impl<'src> ASTNode<'src> for WhileLoop<'src> {
	fn span<'a>(&'a self) -> Span<'src> {
		let start = self._while.start();
		let end = self.block.end();

		let source = self._while.source();
		let file = self._while.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'src> Parse<'src> for WhileLoop<'src> {
	fn parse(input: &ParseStream<'src, '_>) -> ParseResult<'src, Self> {
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
