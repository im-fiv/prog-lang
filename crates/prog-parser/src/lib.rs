// TODO: write a proc macro for implementing ASTNode automatically, where applicable.

mod stream;
pub mod token;
pub mod ast;
pub mod errors;

use anyhow::Result;
pub use prog_utils::pretty_errors::{Position, Span};

pub use stream::ParseStream;
pub use token::Token;

pub trait ASTNode {
	fn span(&self) -> Span;

	fn source(&self) -> &str { self.span().source() }

	fn value(&self) -> &str { self.span().value() }

	fn value_owned(&self) -> String { self.value().to_owned() }

	fn position(&self) -> Position { self.span().position() }

	fn start(&self) -> usize { self.position().start() }

	fn end(&self) -> usize { self.position().end() }
}

pub trait Parse<'inp>: Sized + ASTNode {
	fn parse(input: &ParseStream<'inp>) -> Result<Self>;
}

pub trait ParsePrecedence<'inp>: Parse<'inp> {
	fn parse_precedence(input: &ParseStream<'inp>, precedence: u8) -> Result<Self>;
}
