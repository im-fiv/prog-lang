mod errors;
mod token;
mod stream;

use anyhow::{bail, Result};
pub use errors::{LexError, LexErrorKind};
use prog_utils::pretty_errors::{Position, Span};
pub use stream::TokenStream;
pub use token::{Keyword, Token, TokenKind};

pub fn lex<'inp>(input: &'inp str, file: &'inp str) -> Result<TokenStream<'inp>> {
	let mut chars = input.char_indices().peekable();
	let mut stream = TokenStream::new();

	while let Some((start_index, char)) = chars.next() {
		let kind = match char {
			' ' | '\t' | '\n' | '\r' => continue,

			'+' => TokenKind::Plus,
			'-' => {
				if chars.peek().map_or(false, |(_, next)| *next == '>') {
					chars.next();
					TokenKind::Arrow
				} else {
					TokenKind::Minus
				}
			}
			'*' => TokenKind::Asterisk,
			'/' => TokenKind::Slash,
			'=' => {
				if chars.peek().map_or(false, |(_, next)| *next == '=') {
					chars.next();
					TokenKind::EqEq
				} else {
					TokenKind::Eq
				}
			}
			'.' => TokenKind::Dot,

			'(' => TokenKind::LeftParen,
			')' => TokenKind::RightParen,
			'{' => TokenKind::LeftBrace,
			'}' => TokenKind::RightBrace,
			'[' => TokenKind::LeftBracket,
			']' => TokenKind::RightBracket,

			c if c.is_ascii_alphabetic() || c == '_' => {
				let mut ident = String::new();
				ident.push(c);

				while let Some((_, next_char)) = chars.peek() {
					if !next_char.is_ascii_alphanumeric() && *next_char != '_' {
						break;
					}

					ident.push(*next_char);
					chars.next();
				}

				if let Some(kw) = Keyword::parse(&ident) {
					TokenKind::Keyword(kw)
				} else {
					TokenKind::Identifier
				}
			}

			c if c.is_ascii_digit() => {
				let mut number = String::new();
				number.push(c);

				while let Some((_, next_char)) = chars.peek() {
					if !next_char.is_ascii_digit() {
						break;
					}

					number.push(*next_char);
					chars.next();
				}

				let _ = number.parse::<f64>()?;
				TokenKind::Number
			}

			c => {
				bail!(LexError::new(
					input.to_owned(),
					file.to_owned(),
					Position::new(start_index, start_index + 1),
					LexErrorKind::UnexpectedToken(errors::UnexpectedToken(c))
				))
			}
		};

		let end_index = chars.peek().map_or(input.len(), |(idx, _)| *idx);
		let position = Position::new(start_index, end_index);
		let span = Span::new(input, position);

		stream.push(Token::new(kind, span));
	}

	stream.push(Token::new(
		TokenKind::Eof,
		Span::new(input, Position::new(input.len(), input.len()))
	));

	Ok(stream)
}
