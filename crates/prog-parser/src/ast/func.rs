use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseResult, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Func<'src> {
	pub _func: token::Func<'src>,
	pub _lp: token::LeftParen<'src>,
	pub args: Punctuated<'src, Ident<'src>, token::Comma<'src>>,
	pub _rp: token::RightParen<'src>,
	pub _do: token::Do<'src>,
	pub stmts: Vec<Stmt<'src>>,
	pub _end: token::End<'src>
}

impl<'src> ASTNode<'src> for Func<'src> {
	fn span<'a>(&'a self) -> Span<'src> {
		let start = self._func.start();
		let end = self._end.end();

		let source = self._func.source();
		let file = self._func.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'src> Parse<'src> for Func<'src> {
	fn parse(input: &ParseStream<'src, '_>) -> ParseResult<'src, Self> {
		let _func = input.parse::<token::Func>()?;
		let _lp = input.parse::<token::LeftParen>()?;
		let args = input
			.try_parse::<Punctuated<Ident, token::Comma>>()
			.unwrap_or_default();
		let _rp = input.parse::<token::RightParen>()?;
		let _do = input.parse::<token::Do>()?;
		let mut stmts = vec![];

		while let Ok(stmt) = input.try_parse::<Stmt>() {
			stmts.push(stmt);
		}

		let _end = input.parse::<token::End>()?;

		Ok(Self {
			_func,
			_lp,
			args,
			_rp,
			_do,
			stmts,
			_end
		})
	}
}
