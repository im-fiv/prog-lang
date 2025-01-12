use std::fmt::{self, Debug, Display};

pub enum ProgError<'kind> {
	Lex(prog_lexer::LexError),
	Parse(prog_parser::ParseError),
	Interpret(prog_interpreter::InterpretError<'kind>)
}

impl From<prog_lexer::LexError> for ProgError<'_> {
	fn from(err: prog_lexer::LexError) -> Self { Self::Lex(err) }
}

impl From<prog_parser::ParseError> for ProgError<'_> {
	fn from(err: prog_parser::ParseError) -> Self { Self::Parse(err) }
}

impl<'kind> From<prog_interpreter::InterpretError<'kind>> for ProgError<'kind> {
	fn from(err: prog_interpreter::InterpretError<'kind>) -> Self { Self::Interpret(err) }
}

impl Display for ProgError<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Lex(err) => err as &dyn Display,
			Self::Parse(err) => err as &dyn Display,
			Self::Interpret(err) => err as &dyn Display
		}
		.fmt(f)
	}
}

impl Debug for ProgError<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Lex(err) => err as &dyn Debug,
			Self::Parse(err) => err as &dyn Debug,
			Self::Interpret(err) => err as &dyn Debug
		}
		.fmt(f)
	}
}

impl std::error::Error for ProgError<'_> {}
