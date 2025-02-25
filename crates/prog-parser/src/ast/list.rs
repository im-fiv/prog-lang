use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseResult, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct List<'src> {
	pub _lb: token::LeftBracket<'src>,
	pub items: Box<Punctuated<'src, Expr<'src>, token::Comma<'src>>>,
	pub _rb: token::RightBracket<'src>
}

impl<'src> ASTNode<'src> for List<'src> {
	fn span<'a>(&'a self) -> Span<'src> {
		let start = self._lb.start();
		let end = self._rb.end();

		let source = self._lb.source();
		let file = self._lb.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'src> Parse<'src> for List<'src> {
	fn parse(input: &ParseStream<'src, '_>) -> ParseResult<'src, Self> {
		let _lb = input.parse::<token::LeftBracket>()?;
		let items = input
			.try_parse::<Punctuated<Expr, token::Comma>>()
			.map(Box::new)
			.unwrap_or_default();
		let _rb = input.parse::<token::RightBracket>()?;

		Ok(Self { _lb, items, _rb })
	}
}
