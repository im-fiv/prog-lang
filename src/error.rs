use std::fmt::{self, Debug, Display};

pub enum ProgError {
	Lex(prog_lexer::LexError),
	Parse(prog_parser::ParseError),
	Interpret(prog_interpreter::InterpretError)
}

impl From<prog_lexer::LexError> for ProgError {
	fn from(err: prog_lexer::LexError) -> Self { Self::Lex(err) }
}

impl From<prog_parser::ParseError> for ProgError {
	fn from(err: prog_parser::ParseError) -> Self { Self::Parse(err) }
}

impl From<prog_interpreter::InterpretError> for ProgError {
	fn from(err: prog_interpreter::InterpretError) -> Self { Self::Interpret(err) }
}

impl Display for ProgError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Lex(err) => err as &dyn Display,
			Self::Parse(err) => err as &dyn Display,
			Self::Interpret(err) => err as &dyn Display
		}
		.fmt(f)
	}
}

impl Debug for ProgError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Lex(err) => err as &dyn Debug,
			Self::Parse(err) => err as &dyn Debug,
			Self::Interpret(err) => err as &dyn Debug
		}
		.fmt(f)
	}
}

impl std::error::Error for ProgError {}
