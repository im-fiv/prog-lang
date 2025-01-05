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
}

pub trait Parse<'inp>: Sized + ASTNode {
	fn parse(input: &ParseStream<'inp>) -> Result<Self>;
}

pub trait ParsePrecedence<'inp>: Parse<'inp> {
	fn parse_precedence(input: &ParseStream<'inp>, precedence: u8) -> Result<Self>;
}
