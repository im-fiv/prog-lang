use std::fmt::{self, Debug};

use prog_lexer::TokenKind;

use crate::{error, ASTNode, Parse, ParseError, ParseErrorKind, ParseResult, ParseStream, Span};

#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Lit<'src> {
	pub kind: LitKind,
	pub span: Span<'src>
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum LitKind {
	Num(f64),
	Bool(bool),
	Str(String),
	None
}

impl Lit<'_> {
	pub fn strip_quotes(str: &str) -> &str { str.trim_start_matches('\"').trim_end_matches('\"') }
}

impl<'src> ASTNode<'src> for Lit<'src> {
	fn span<'a>(&'a self) -> Span<'src> { self.span }
}

impl<'src> Parse<'src> for Lit<'src> {
	fn parse(input: &ParseStream<'src>) -> ParseResult<'src, Self> {
		let token = input.expect_next()?;
		let span = token.span();

		match token.kind() {
			TokenKind::Number => {
				let num = token.value().parse::<f64>().unwrap();

				Ok(Self {
					kind: LitKind::Num(num),
					span
				})
			}

			TokenKind::True => {
				Ok(Self {
					kind: LitKind::Bool(true),
					span
				})
			}

			TokenKind::False => {
				Ok(Self {
					kind: LitKind::Bool(false),
					span
				})
			}

			TokenKind::String => {
				let str = Self::strip_quotes(token.value()).to_owned();

				Ok(Self {
					kind: LitKind::Str(str),
					span
				})
			}

			TokenKind::None => {
				Ok(Self {
					kind: LitKind::None,
					span
				})
			}

			kind => {
				Err(ParseError::new(
					span,
					ParseErrorKind::Internal(error::Internal(format!(
						"unknown literal `{token}` of type `{kind:?}`"
					)))
				))
			}
		}
	}
}

impl Debug for Lit<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut s = f.debug_tuple("Lit");

		let value = match &self.kind {
			LitKind::Num(lit) => lit as &dyn Debug,
			LitKind::Bool(lit) => lit as &dyn Debug,
			LitKind::Str(lit) => lit as &dyn Debug,
			LitKind::None => &"none" as &dyn Debug
		};

		s.field(value);
		s.finish()
	}
}
