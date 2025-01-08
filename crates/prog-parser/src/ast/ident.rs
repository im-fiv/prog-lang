use std::fmt::{self, Debug, Display};

use crate::{token, ASTNode, Parse, ParseResult, ParseStream, Span};

#[derive(Clone, Copy, PartialEq)]
pub struct Ident<'inp> {
	_ident: token::Ident<'inp>
}

impl ASTNode for Ident<'_> {
	fn span(&self) -> Span { self._ident.span() }
}

impl<'inp> Parse<'inp> for Ident<'inp> {
	fn parse(input: &ParseStream<'inp>) -> ParseResult<Self> {
		Ok(Self {
			_ident: input.parse::<token::Ident>()?
		})
	}
}

impl Display for Ident<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self._ident.value())
	}
}

impl Debug for Ident<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut s = f.debug_tuple("Ident");
		s.field(&self._ident.value());
		s.finish()
	}
}
