use std::fmt::{self, Display};

use crate::{Position, Span};

#[derive(Debug)]
pub struct TokenStream<'inp> {
	buffer: Vec<Token<'inp>>
}

impl<'inp> TokenStream<'inp> {
	pub fn new() -> Self { Self { buffer: vec![] } }

	pub fn buffer(&'inp self) -> &'inp [Token<'inp>] { &self.buffer }

	pub fn unwrap(self) -> Vec<Token<'inp>> { self.buffer }

	pub(crate) fn push(&mut self, token: Token<'inp>) { self.buffer.push(token); }
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
	fn default() -> Self { Self::new() }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Token<'inp> {
	kind: TokenKind,
	span: Span<'inp>
}

impl<'inp> Token<'inp> {
	pub fn new(kind: TokenKind, span: Span<'inp>) -> Self { Self { kind, span } }

	pub fn kind(&self) -> TokenKind { self.kind }

	pub fn span(&self) -> Span<'inp> { self.span }

	pub fn position(&self) -> Position { self.span().position() }

	pub fn value(&self) -> &'inp str { self.span().value() }
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
	// Keywords
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
	/// `elseif`
	ElseIf,
	/// `else`
	Else,
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
	Extern,

	// Special tokens
	Ident,
	Comment,
	Number,
	String,

	// Operator tokens
	/// `+`
	Plus,
	/// `-`
	Minus,
	/// `*`
	Asterisk,
	/// `/`
	Slash,
	/// `%`
	Sign,
	/// `=`
	Eq,
	/// `==`
	EqEq,
	/// `!=`
	Neq,
	/// `->`
	Arrow,
	/// `=>`
	FatArrow,
	/// `.`
	Dot,
	/// `,`
	Comma,

	/// `>`
	Gt,
	/// `<`
	Lt,
	/// `>=`
	Gte,
	/// `<=`
	Lte,

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

impl TokenKind {
	pub fn as_keyword(input: &str) -> Option<Self> {
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
			"elseif" => Some(Self::ElseIf),
			"else" => Some(Self::Else),
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
