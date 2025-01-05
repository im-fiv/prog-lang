mod stream;
pub mod token;
pub mod ast;
pub mod errors;

use anyhow::Result;
pub use prog_utils::pretty_errors::{Position, Span};
pub use stream::ParseStream;
pub use token::Token;

pub trait ASTNode<'inp> {
	fn span(&'inp self) -> Span<'inp>;
}

pub trait Parse<'inp>: Sized + ASTNode<'inp> {
	fn parse(input: &'_ ParseStream<'inp>) -> Result<Self>;
}

pub trait ParsePrecedence<'inp>: Parse<'inp> {
	fn parse_precedence(input: &'_ ParseStream<'inp>, precedence: u8) -> Result<Self>;
}
