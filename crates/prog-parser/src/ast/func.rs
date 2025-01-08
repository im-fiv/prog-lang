use crate::ast::*;
use crate::{token, ParseResult, ASTNode, Parse, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct Func<'inp> {
	pub _func: token::Func<'inp>,
	pub _lp: token::LeftParen<'inp>,
	pub args: Option<Punctuated<'inp, Ident<'inp>, token::Comma<'inp>>>,
	pub _rp: token::RightParen<'inp>,
	pub _do: token::Do<'inp>,
	pub stmts: Vec<Statement<'inp>>,
	pub _end: token::End<'inp>
}

impl ASTNode for Func<'_> {
	fn span(&self) -> Span {
		let start = self._func.start();
		let end = self._end.end();

		let source = self._func.source();
		let file = self._func.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'inp> Parse<'inp> for Func<'inp> {
	fn parse(input: &ParseStream<'inp>) -> ParseResult<Self> {
		let _func = input.parse::<token::Func>()?;
		let _lp = input.parse::<token::LeftParen>()?;
		let args = input
			.try_parse::<Punctuated<'inp, Ident, token::Comma>>()
			.ok();
		let _rp = input.parse::<token::RightParen>()?;
		let _do = input.parse::<token::Do>()?;
		let mut stmts = vec![];

		while let Ok(stmt) = input.try_parse::<Statement>() {
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
