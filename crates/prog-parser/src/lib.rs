// TODO: write a derive macro for implementing ASTNode automatically, where applicable.
// TODO: implement contexts for errors as the current state of error reporting is unacceptable.

mod stream;
pub mod token;
pub mod ast;
pub mod error;

pub use error::{ParseError, ParseErrorKind};
pub use stream::ParseStream;
pub use token::Token;

pub use prog_utils::pretty_errors::{Position, Span};

pub type ParseResult<'s, T> = Result<T, ParseError<'s>>;

pub trait ASTNode<'src> {
	fn span<'a>(&'a self) -> Span<'src>;

	fn source<'a>(&'a self) -> &'src str { self.span().source() }
	fn file<'a>(&'a self) -> &'src str { self.span().file() }
	fn position(&self) -> Position { self.span().position() }
	fn start(&self) -> usize { self.position().start() }
	fn end(&self) -> usize { self.position().end() }
	fn value<'a>(&'a self) -> &'src str { self.span().value() }
	fn value_owned(&self) -> String { self.value().to_owned() }
}

pub trait Parse<'src>: Sized + ASTNode<'src> {
	fn parse(input: &ParseStream<'src>) -> ParseResult<'src, Self>;
}

pub trait ParsePrecedence<'src>: Parse<'src> {
	fn parse_precedence(input: &ParseStream<'src>, precedence: u8) -> ParseResult<'src, Self>;
}
