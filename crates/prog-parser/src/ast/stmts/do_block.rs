use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseResult, ParseStream, Position, Span};

use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
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

#[cfg(feature = "serde")]
impl serde::Serialize for DoBlock<'_> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer
	{
		use serde::ser::SerializeStruct;

		let mut s = serializer.serialize_struct("DoBlock", 3)?;
		s.serialize_field("_do", &self._do)?;
		s.serialize_field("ast", self.stmts.as_ref())?;
		s.serialize_field("_end", &self._end)?;
		s.end()
	}
}
