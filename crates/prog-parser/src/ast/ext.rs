use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseResult, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Extern<'src> {
	pub _extern: token::Extern<'src>,
	pub value: Box<Expr<'src>>
}

impl<'src> ASTNode<'src> for Extern<'src> {
	fn span<'a>(&'a self) -> Span<'src> {
		let start = self._extern.start();
		let end = self.value.end();

		let source = self._extern.source();
		let file = self._extern.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'src> Parse<'src> for Extern<'src> {
	fn parse(input: &ParseStream<'src, '_>) -> ParseResult<'src, Self> {
		let _extern = input.parse::<token::Extern>()?;
		let value = Box::new(input.parse::<Expr>()?);

		Ok(Self { _extern, value })
	}
}
