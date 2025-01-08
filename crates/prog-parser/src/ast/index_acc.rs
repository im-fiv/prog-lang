use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseResult, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct IndexAcc<'inp> {
	pub list: Box<Term<'inp>>,
	pub _lb: token::LeftBracket<'inp>,
	pub index: Box<Expr<'inp>>,
	pub _rb: token::RightBracket<'inp>
}

impl<'inp> IndexAcc<'inp> {
	pub fn parse_with_list(input: &ParseStream<'inp>, list: Box<Term<'inp>>) -> ParseResult<Self> {
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

impl ASTNode for IndexAcc<'_> {
	fn span(&self) -> Span {
		let start = self.list.start();
		let end = self._rb.end();

		let source = self.list.source();
		let file = self.list.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'inp> Parse<'inp> for IndexAcc<'inp> {
	fn parse(input: &ParseStream<'inp>) -> ParseResult<Self> {
		// To support chained operations or complex index access expressions
		// we have to rely on `Term`'s implementation
		Term::parse_variant::<Self>(input)
	}
}
