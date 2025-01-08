// TODO: write a derive macro for implementing ASTNode automatically, where applicable.
// TODO: implement contexts for errors as the current state of error reporting is unacceptable.

mod stream;
pub mod token;
pub mod ast;
pub mod errors;

pub use stream::ParseStream;
pub use token::Token;
pub use errors::{ParseError, ParseErrorKind};

pub use prog_utils::pretty_errors::{Position, Span};

pub type ParseResult<T> = Result<T, ParseError>;

pub trait ASTNode {
	fn span(&self) -> Span;

	fn source(&self) -> &str { self.span().source() }
	fn file(&self) -> &str { self.span().file() }
	fn value(&self) -> &str { self.span().value() }
	fn value_owned(&self) -> String { self.value().to_owned() }
	fn position(&self) -> Position { self.span().position() }
	fn start(&self) -> usize { self.position().start() }
	fn end(&self) -> usize { self.position().end() }
}

pub trait Parse<'src>: Sized + ASTNode {
	fn parse(input: &ParseStream<'src>) -> ParseResult<Self>;
}

pub trait ParsePrecedence<'src>: Parse<'src> {
	fn parse_precedence(input: &ParseStream<'src>, precedence: u8) -> ParseResult<Self>;
}
