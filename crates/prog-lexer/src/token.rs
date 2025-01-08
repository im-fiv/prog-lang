use std::fmt::{self, Display};

use crate::{Position, Span};

#[derive(Debug)]
pub struct TokenStream<'src> {
	buffer: Vec<Token<'src>>
}

impl<'src> TokenStream<'src> {
	pub fn new() -> Self { Self { buffer: vec![] } }

	pub fn buffer(&'src self) -> &'src [Token<'src>] { &self.buffer }

	pub fn unwrap(self) -> Vec<Token<'src>> { self.buffer }

	pub(crate) fn push(&mut self, token: Token<'src>) { self.buffer.push(token); }

	pub(crate) fn filter_comments(&mut self) {
		self.buffer.retain(|t| t.kind() != TokenKind::Comment);
	}
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
pub struct Token<'src> {
	kind: TokenKind,
	span: Span<'src>
}

impl<'src> Token<'src> {
	pub fn new(kind: TokenKind, span: Span<'src>) -> Self { Self { kind, span } }

	pub fn kind(&self) -> TokenKind { self.kind }

	pub fn span(&self) -> Span<'src> { self.span }

	pub fn position(&self) -> Position { self.span().position() }

	pub fn value(&self) -> &'src str { self.span().value() }
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

impl Display for TokenKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::True => write!(f, "true"),
			Self::False => write!(f, "false"),
			Self::Def => write!(f, "def"),
			Self::Func => write!(f, "func"),
			Self::Do => write!(f, "do"),
			Self::End => write!(f, "end"),
			Self::Return => write!(f, "return"),
			Self::While => write!(f, "while"),
			Self::Break => write!(f, "break"),
			Self::Continue => write!(f, "continue"),
			Self::If => write!(f, "if"),
			Self::Then => write!(f, "then"),
			Self::ElseIf => write!(f, "elseif"),
			Self::Else => write!(f, "else"),
			Self::None => write!(f, "none"),
			Self::And => write!(f, "and"),
			Self::Or => write!(f, "or"),
			Self::Not => write!(f, "not"),
			Self::Class => write!(f, "class"),
			Self::Extern => write!(f, "extern"),
			Self::Ident => write!(f, "identifier"),
			Self::Comment => write!(f, "comment"),
			Self::Number => write!(f, "number"),
			Self::String => write!(f, "string"),
			Self::Plus => write!(f, "+"),
			Self::Minus => write!(f, "-"),
			Self::Asterisk => write!(f, "*"),
			Self::Slash => write!(f, "/"),
			Self::Sign => write!(f, "%"),
			Self::Eq => write!(f, "="),
			Self::EqEq => write!(f, "=="),
			Self::Neq => write!(f, "!="),
			Self::Arrow => write!(f, "->"),
			Self::FatArrow => write!(f, "=>"),
			Self::Dot => write!(f, "."),
			Self::Comma => write!(f, ","),
			Self::Gt => write!(f, ">"),
			Self::Lt => write!(f, "<"),
			Self::Gte => write!(f, ">="),
			Self::Lte => write!(f, "<="),
			Self::LeftParen => write!(f, "("),
			Self::RightParen => write!(f, ")"),
			Self::LeftBrace => write!(f, "{{"),
			Self::RightBrace => write!(f, "}}"),
			Self::LeftBracket => write!(f, "["),
			Self::RightBracket => write!(f, "]"),
			Self::Eof => write!(f, "<EOF>")
		}
	}
}
