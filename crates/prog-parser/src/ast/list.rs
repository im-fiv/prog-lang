use crate::ast::*;
use crate::{token, ParseResult, ASTNode, Parse, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct List<'inp> {
	pub _lb: token::LeftBracket<'inp>,
	pub items: Option<Box<Punctuated<'inp, Expr<'inp>, token::Comma<'inp>>>>,
	pub _rb: token::RightBracket<'inp>
}

impl ASTNode for List<'_> {
	fn span(&self) -> Span {
		let start = self._lb.start();
		let end = self._rb.end();

		let source = self._lb.source();
		let file = self._lb.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'inp> Parse<'inp> for List<'inp> {
	fn parse(input: &ParseStream<'inp>) -> ParseResult<Self> {
		let _lb = input.parse::<token::LeftBracket>()?;
		let items = input
			.try_parse::<Punctuated<'inp, Expr, token::Comma>>()
			.map(Box::new)
			.ok();
		let _rb = input.parse::<token::RightBracket>()?;

		Ok(Self { _lb, items, _rb })
	}
}
