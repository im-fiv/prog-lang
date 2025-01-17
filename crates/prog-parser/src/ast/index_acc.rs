use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseResult, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct IndexAcc<'src> {
	pub list: Box<Term<'src>>,
	pub _lb: token::LeftBracket<'src>,
	pub index: Box<Expr<'src>>,
	pub _rb: token::RightBracket<'src>
}

impl<'src> IndexAcc<'src> {
	pub fn parse_with_list(input: &ParseStream<'src>, list: Box<Term<'src>>) -> ParseResult<'src, Self> {
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

impl<'src> ASTNode<'src> for IndexAcc<'src> {
	fn span<'a>(&'a self) -> Span<'src> {
		let start = self.list.start();
		let end = self._rb.end();

		let source = self.list.source();
		let file = self.list.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'src> Parse<'src> for IndexAcc<'src> {
	fn parse(input: &ParseStream<'src>) -> ParseResult<'src, Self> {
		// To support chained operations or complex index access expressions
		// we have to rely on `Term`'s implementation
		Term::parse_variant::<Self>(input)
	}
}
