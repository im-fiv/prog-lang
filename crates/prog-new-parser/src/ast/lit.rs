use std::fmt::{self, Debug};

use anyhow::{bail, Result};
use prog_lexer::TokenKind;

use crate::{ASTNode, Parse, ParseStream, Span};

#[derive(Clone, PartialEq)]
pub struct Lit<'inp> {
	pub kind: LitKind,
	pub span: Span<'inp>
}

#[derive(Debug, Clone, PartialEq)]
pub enum LitKind {
	Number(f64),
	Boolean(bool),
	String(String)
}

impl Lit<'_> {
	pub fn strip_quotes(str: &str) -> &str { str.trim_start_matches('\"').trim_end_matches('\"') }
}

impl ASTNode for Lit<'_> {
	fn span(&self) -> Span { self.span }
}

impl<'inp> Parse<'inp> for Lit<'inp> {
	fn parse(input: &'_ ParseStream<'inp>) -> Result<Self> {
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

			// TODO: proper error reporting
			kind => bail!("Unknown literal `{token}` of type `{kind:?}`")
		}
	}
}

impl Debug for Lit<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut s = f.debug_tuple("Lit");

		let value = match &self.kind {
			LitKind::Number(lit) => lit as &dyn Debug,
			LitKind::Boolean(lit) => lit as &dyn Debug,
			LitKind::String(lit) => lit as &dyn Debug
		};

		s.field(value);
		s.finish()
	}
}
