use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseResult, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct Call<'inp> {
	pub func: Box<Term<'inp>>,
	pub _lp: token::LeftParen<'inp>,
	pub args: Option<Box<Punctuated<'inp, Expr<'inp>, token::Comma<'inp>>>>,
	pub _rp: token::RightParen<'inp>
}

impl<'inp> Call<'inp> {
	pub fn parse_with_func(input: &ParseStream<'inp>, func: Box<Term<'inp>>) -> ParseResult<Self> {
		let _lp = input.parse::<token::LeftParen>()?;
		let args = input
			.try_parse::<Punctuated<'inp, Expr, token::Comma>>()
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

impl<'inp> Parse<'inp> for Call<'inp> {
	fn parse(input: &ParseStream<'inp>) -> ParseResult<Self> {
		// To support chained operations or complex call expressions
		// we have to rely on `Term`'s implementation
		Term::parse_variant::<Self>(input)
	}
}
