use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseResult, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct DoBlock<'inp> {
	pub _do: token::Do<'inp>,
	pub stmts: Vec<Statement<'inp>>,
	pub _end: token::End<'inp>
}

impl ASTNode for DoBlock<'_> {
	fn span(&self) -> Span {
		let start = self._do.start();
		let end = self._end.end();

		let source = self._do.source();
		let file = self._do.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'inp> Parse<'inp> for DoBlock<'inp> {
	fn parse(input: &ParseStream<'inp>) -> ParseResult<Self> {
		let _do = input.parse::<token::Do>()?;

		let mut stmts = vec![];
		while let Ok(stmt) = input.try_parse::<Statement>() {
			stmts.push(stmt);
		}

		let _end = input.parse::<token::End>()?;

		Ok(Self { _do, stmts, _end })
	}
}
