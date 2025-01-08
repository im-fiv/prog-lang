use crate::{token, ASTNode, Parse, ParseResult, ParseStream, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct Break<'src> {
	_break: token::Break<'src>
}

#[derive(Debug, Clone, PartialEq)]
pub struct Continue<'src> {
	_continue: token::Continue<'src>
}

impl ASTNode for Break<'_> {
	fn span(&self) -> Span { self._break.span() }
}

impl<'src> Parse<'src> for Break<'src> {
	fn parse(input: &ParseStream<'src>) -> ParseResult<Self> {
		let _break = input.parse::<token::Break>()?;

		Ok(Self { _break })
	}
}

impl ASTNode for Continue<'_> {
	fn span(&self) -> Span { self._continue.span() }
}

impl<'src> Parse<'src> for Continue<'src> {
	fn parse(input: &ParseStream<'src>) -> ParseResult<Self> {
		let _continue = input.parse::<token::Continue>()?;

		Ok(Self { _continue })
	}
}
