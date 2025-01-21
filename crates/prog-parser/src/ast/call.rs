use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseResult, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Call<'src> {
	pub callee: Box<Term<'src>>,
	pub _lp: token::LeftParen<'src>,
	pub args: Box<Punctuated<'src, Expr<'src>, token::Comma<'src>>>,
	pub _rp: token::RightParen<'src>
}

impl<'src> Call<'src> {
	pub fn parse_with_callee(input: &ParseStream<'src, '_>, callee: Box<Term<'src>>) -> ParseResult<'src, Self> {
		let _lp = input.parse::<token::LeftParen>()?;
		let args = input
			.try_parse::<Punctuated<Expr, token::Comma>>()
			.map(Box::new)
			.unwrap_or_default();
		let _rp = input.parse::<token::RightParen>()?;

		Ok(Self {
			callee,
			_lp,
			args,
			_rp
		})
	}
}

impl<'src> ASTNode<'src> for Call<'src> {
	fn span<'a>(&'a self) -> Span<'src> {
		let start = self.callee.start();
		let end = self._rp.end();

		let source = self.callee.source();
		let file = self.callee.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'src> Parse<'src> for Call<'src> {
	fn parse(input: &ParseStream<'src, '_>) -> ParseResult<'src, Self> {
		// To support chained operations or complex call expressions
		// we have to rely on `Term`'s implementation
		Term::parse_variant::<Self>(input)
	}
}
