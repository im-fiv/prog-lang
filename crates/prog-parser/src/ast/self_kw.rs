use std::fmt::{self, Debug, Display};

use crate::ast::*;
use crate::{ASTNode, Parse, ParseError, ParseResult, ParseStream, Span};

#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SelfKw<'src> {
	_ident: Ident<'src>
}

impl SelfKw<'_> {
	pub const KEYWORD: &'static str = "self";
}

impl<'src> ASTNode<'src> for SelfKw<'src> {
	fn span<'a>(&'a self) -> Span<'src> { self._ident.span() }
}

impl<'src> Parse<'src> for SelfKw<'src> {
	fn parse(input: &ParseStream<'src, '_>) -> ParseResult<'src, Self> {
		let _ident = input.parse::<Ident>()?;

		if _ident.value() != Self::KEYWORD {
			return Err(ParseError::new(
				_ident.span(),
				crate::ParseErrorKind::Internal(crate::error::Internal(format!(
					"tried to parse keyword `{}` but found identifier `{}`",
					Self::KEYWORD,
					_ident
				)))
			));
		}

		Ok(Self { _ident })
	}
}

impl Display for SelfKw<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self._ident.value())
	}
}

impl Debug for SelfKw<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut s = f.debug_tuple("SelfKw");
		s.field(&self._ident.value());
		s.finish()
	}
}
