use std::fmt::{self, Debug};

use anyhow::Result;

use crate::{token, ASTNode, Parse, ParseStream, Span};

#[derive(Clone, Copy, PartialEq)]
pub struct Ident<'inp> {
	_ident: token::Ident<'inp>
}

impl ASTNode for Ident<'_> {
	fn span(&self) -> Span { self._ident.span() }
}

impl<'inp> Parse<'inp> for Ident<'inp> {
	fn parse(input: &ParseStream<'inp>) -> Result<Self> {
		Ok(Self {
			_ident: input.parse::<token::Ident>()?
		})
	}
}

impl ToString for Ident<'_> {
	fn to_string(&self) -> String { self._ident.value_owned() }
}

impl Debug for Ident<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut s = f.debug_tuple("Ident");
		s.field(&self._ident);
		s.finish()
	}
}
