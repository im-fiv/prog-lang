use anyhow::{bail, Result};

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
		use prog_lexer::TokenKind;

		// TODO: error handling
		let token = input.next().unwrap();
		let span = token.span();

		match token.kind() {
			TokenKind::Number => {
				Ok(Self {
					kind: LitKind::Number(
						// TODO: error handling
						token.value().parse::<f64>().unwrap()
					),
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

			kind => bail!("Unknown literal `{token}` of type `{kind:?}`")
		}
	}
}
