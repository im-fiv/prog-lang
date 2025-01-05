use anyhow::{bail, Result};
use prog_lexer::TokenKind;

use crate::{ASTNode, Parse, ParseStream, Span};

#[derive(Debug, Clone, PartialEq)]
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

impl ASTNode<'_> for Lit<'_> {
	fn span(&self) -> Span { self.span }
}

impl<'inp> Parse<'inp> for Lit<'inp> {
	fn parse(input: &ParseStream<'inp>) -> Result<Self> {
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
				Ok(Self {
					kind: LitKind::String(
						// TODO: strip quotes
						token.value().to_owned()
					),
					span
				})
			}

			// TODO: proper error reporting
			kind => bail!("Unknown literal `{token}` of type `{kind:?}`")
		}
	}
}
