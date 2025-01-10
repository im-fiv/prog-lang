use std::fmt::{self, Debug, Display};

use crate::{token, ASTNode, Parse, ParseResult, ParseStream, Span};

#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Ident<'src> {
	_ident: token::Ident<'src>
}

impl<'src> ASTNode<'src> for Ident<'src> {
	fn span<'a>(&'a self) -> Span<'src> { self._ident.span() }
}

impl<'src> Parse<'src> for Ident<'src> {
	fn parse(input: &ParseStream<'src>) -> ParseResult<Self> {
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
