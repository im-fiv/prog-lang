use std::fmt::{self, Display};

use ariadne::Span as _;

use crate::Span;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Token<'inp> {
	kind: TokenKind,
	span: Span<'inp>
}

impl<'inp> Token<'inp> {
	pub fn new(kind: TokenKind, span: Span<'inp>) -> Self {
		Self { kind, span }
	}

	pub fn start(&self) -> usize {
		self.span.start()
	}

	pub fn end(&self) -> usize {
		self.span.end()
	}

	pub fn value(&self) -> &'inp str {
		self.span.value()
	}
}

impl Display for Token<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.kind == TokenKind::Eof {
			return Ok(());
		}

		write!(f, "{}", self.span.value())
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
	Identifier,
	Keyword(Keyword),
	Number,

	/// `+`
	Plus,
	/// `-`
	Minus,
	/// `*`
	Asterisk,
	/// `/`
	Slash,
	/// `=`
	Eq,
	/// `==`
	EqEq,
	/// `->`
	Arrow,
	/// `.`
	Dot,

	/// `(`
	LeftParen,
	/// `)`
	RightParen,
	/// `{`
	LeftBrace,
	/// `}`
	RightBrace,
	/// `[`
	LeftBracket,
	/// `]`
	RightBracket,

	/// End-of-file
	Eof
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Keyword {
	/// `true`
	True,
	/// `false`
	False,
	/// `def`
	Def,
	/// `func`
	Func,
	/// `do`
	Do,
	/// `end`
	End,
	/// `return`
	Return,
	/// `while`
	While,
	/// `break`
	Break,
	/// `continue`
	Continue,
	/// `if`
	If,
	/// `then`
	Then,
	/// `none`
	None,
	/// `and`
	And,
	/// `or`
	Or,
	/// `not`
	Not,
	/// `class`
	Class,
	/// `extern`
	Extern
}

impl Keyword {
	pub fn parse(input: &str) -> Option<Self> {
		match input {
			"true" => Some(Self::True),
			"false" => Some(Self::False),
			"def" => Some(Self::Def),
			"func" => Some(Self::Func),
			"do" => Some(Self::Do),
			"end" => Some(Self::End),
			"return" => Some(Self::Return),
			"while" => Some(Self::While),
			"break" => Some(Self::Break),
			"continue" => Some(Self::Continue),
			"if" => Some(Self::If),
			"then" => Some(Self::Then),
			"none" => Some(Self::None),
			"and" => Some(Self::And),
			"or" => Some(Self::Or),
			"not" => Some(Self::Not),
			"class" => Some(Self::Class),
			"extern" => Some(Self::Extern),

			_ => None
		}
	}
}
