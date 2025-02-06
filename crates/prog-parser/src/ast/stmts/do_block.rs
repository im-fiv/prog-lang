use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseResult, ParseStream, Position, Span};

use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct DoBlock<'src> {
	pub _do: token::Do<'src>,
	pub stmts: Rc<[Stmt<'src>]>,
	pub _end: token::End<'src>
}

impl<'src> ASTNode<'src> for DoBlock<'src> {
	fn span<'a>(&'a self) -> Span<'src> {
		let start = self._do.start();
		let end = self._end.end();

		let source = self._do.source();
		let file = self._do.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'src> Parse<'src> for DoBlock<'src> {
	fn parse(input: &ParseStream<'src, '_>) -> ParseResult<'src, Self> {
		let _do = input.parse::<token::Do>()?;

		let mut stmts = vec![];
		while let Ok(stmt) = input.try_parse::<Stmt>() {
			stmts.push(stmt);
		}

		let _end = input.parse::<token::End>()?;

		Ok(Self {
			_do,
			stmts: stmts.into(),
			_end
		})
	}
}
