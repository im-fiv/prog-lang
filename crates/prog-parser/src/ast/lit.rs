use std::fmt::{self, Debug};

use prog_lexer::TokenKind;

use crate::{error, ASTNode, Parse, ParseError, ParseErrorKind, ParseResult, ParseStream, Span};

#[derive(Clone, PartialEq)]
pub struct Lit<'src> {
	pub kind: LitKind,
	pub span: Span<'src>
}

#[derive(Debug, Clone, PartialEq)]
pub enum LitKind {
	Number(f64),
	Boolean(bool),
	String(String),
	None
}

impl Lit<'_> {
	pub fn strip_quotes(str: &str) -> &str { str.trim_start_matches('\"').trim_end_matches('\"') }
}

impl ASTNode for Lit<'_> {
	fn span(&self) -> Span { self.span }
}

impl<'src> Parse<'src> for Lit<'src> {
	fn parse(input: &ParseStream<'src>) -> ParseResult<Self> {
		let token = input.expect_next()?;
		let span = token.span();

		match token.kind() {
			TokenKind::Number => {
				let num = token.value().parse::<f64>().unwrap();

				Ok(Self {
					kind: LitKind::Number(num),
					span
				})
			}

			TokenKind::True => {
				Ok(Self {
					kind: LitKind::Boolean(true),
					span
				})
			}

			TokenKind::False => {
				Ok(Self {
					kind: LitKind::Boolean(false),
					span
				})
			}

			TokenKind::String => {
				let str = Self::strip_quotes(token.value()).to_owned();

				Ok(Self {
					kind: LitKind::String(str),
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
				Err(ParseError::with_span(
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
			LitKind::Number(lit) => lit as &dyn Debug,
			LitKind::Boolean(lit) => lit as &dyn Debug,
			LitKind::String(lit) => lit as &dyn Debug,
			LitKind::None => &"none" as &dyn Debug
		};

		s.field(value);
		s.finish()
	}
}
