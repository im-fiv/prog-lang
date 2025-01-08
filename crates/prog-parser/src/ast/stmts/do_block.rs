use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseResult, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct DoBlock<'src> {
	pub _do: token::Do<'src>,
	pub stmts: Vec<Stmt<'src>>,
	pub _end: token::End<'src>
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

impl<'src> Parse<'src> for DoBlock<'src> {
	fn parse(input: &ParseStream<'src>) -> ParseResult<Self> {
		let _do = input.parse::<token::Do>()?;

		let mut stmts = vec![];
		while let Ok(stmt) = input.try_parse::<Stmt>() {
			stmts.push(stmt);
		}

		let _end = input.parse::<token::End>()?;

		Ok(Self { _do, stmts, _end })
	}
}
