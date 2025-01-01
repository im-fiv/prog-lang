use std::fmt::{self, Display};
use crate::Token;

#[derive(Debug)]
pub struct TokenStream<'inp> {
	pub(crate) buffer: Vec<Token<'inp>>
}

impl<'inp> TokenStream<'inp> {
	pub fn new() -> Self {
		Self {
			buffer: vec![]
		}
	}

	pub fn push(&mut self, token: Token<'inp>) {
		self.buffer.push(token);
	}
}

impl Display for TokenStream<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for token in &self.buffer {
			write!(f, "{token} ")?;
		}

		Ok(())
	}
}

impl Default for TokenStream<'_> {
	fn default() -> Self {
		Self::new()
	}
}