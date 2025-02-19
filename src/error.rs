use std::fmt::{self, Debug, Display};

pub enum ProgError<'s> {
	Lex(prog_lexer::LexError<'s>),
	Parse(prog_parser::ParseError<'s>),
	Interpret(prog_interpreter::InterpretError<'s>)
}

impl<'s> From<prog_lexer::LexError<'s>> for ProgError<'s> {
	fn from(err: prog_lexer::LexError<'s>) -> Self { Self::Lex(err) }
}

impl<'s> From<prog_parser::ParseError<'s>> for ProgError<'s> {
	fn from(err: prog_parser::ParseError<'s>) -> Self { Self::Parse(err) }
}

impl<'s> From<prog_interpreter::InterpretError<'s>> for ProgError<'s> {
	fn from(err: prog_interpreter::InterpretError<'s>) -> Self { Self::Interpret(err) }
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
