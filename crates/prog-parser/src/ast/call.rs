use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseResult, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct Call<'src> {
	pub func: Box<Term<'src>>,
	pub _lp: token::LeftParen<'src>,
	pub args: Option<Box<Punctuated<'src, Expr<'src>, token::Comma<'src>>>>,
	pub _rp: token::RightParen<'src>
}

impl<'src> Call<'src> {
	pub fn parse_with_func(input: &ParseStream<'src>, func: Box<Term<'src>>) -> ParseResult<Self> {
		let _lp = input.parse::<token::LeftParen>()?;
		let args = input
			.try_parse::<Punctuated<'src, Expr, token::Comma>>()
			.map(Box::new)
			.ok();
		let _rp = input.parse::<token::RightParen>()?;

		Ok(Self {
			func,
			_lp,
			args,
			_rp
		})
	}
}

impl ASTNode for Call<'_> {
	fn span(&self) -> Span {
		let start = self.func.start();
		let end = self._rp.end();

		let source = self.func.source();
		let file = self.func.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'src> Parse<'src> for Call<'src> {
	fn parse(input: &ParseStream<'src>) -> ParseResult<Self> {
		// To support chained operations or complex call expressions
		// we have to rely on `Term`'s implementation
		Term::parse_variant::<Self>(input)
	}
}
