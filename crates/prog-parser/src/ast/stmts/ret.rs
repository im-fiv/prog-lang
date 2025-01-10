use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseResult, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Return<'src> {
	pub _return: token::Return<'src>,
	pub value: Expr<'src>
}

impl<'src> ASTNode<'src> for Return<'src> {
	fn span<'a>(&'a self) -> Span<'src> {
		let start = self._return.start();
		let end = self.value.end();

		let source = self._return.source();
		let file = self._return.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'src> Parse<'src> for Return<'src> {
	fn parse(input: &ParseStream<'src>) -> ParseResult<Self> {
		let _return = input.parse::<token::Return>()?;
		let value = input.parse::<Expr>()?;

		Ok(Self { _return, value })
	}
}
