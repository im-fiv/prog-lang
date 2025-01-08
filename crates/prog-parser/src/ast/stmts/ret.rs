use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseResult, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct Return<'inp> {
	pub _return: token::Return<'inp>,
	pub value: Expr<'inp>
}

impl ASTNode for Return<'_> {
	fn span(&self) -> Span {
		let start = self._return.start();
		let end = self.value.end();

		let source = self._return.source();
		let file = self._return.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'inp> Parse<'inp> for Return<'inp> {
	fn parse(input: &ParseStream<'inp>) -> ParseResult<Self> {
		let _return = input.parse::<token::Return>()?;
		let value = input.parse::<Expr>()?;

		Ok(Self { _return, value })
	}
}
