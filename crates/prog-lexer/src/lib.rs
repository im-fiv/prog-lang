mod errors;
mod token;
mod stream;

use anyhow::{bail, Result};
pub use errors::{LexError, LexErrorKind};
use prog_utils::pretty_errors::{Position, Span};
pub use stream::{IteratorConvertion, ParseStream, PeekableWrapper};
pub use token::{Keyword, Token, TokenKind, TokenStream};

pub fn lex<'inp>(source: &'inp str, file: &'inp str) -> Result<TokenStream<'inp>> {
	let mut ps = ParseStream::new(source, file);
	let mut ts = TokenStream::new();

	while let Some((start_index, char)) = ps.next() {
		let kind = match char {
			' ' | '\t' | '\n' | '\r' => continue,

			'+' => TokenKind::Plus,
			'-' => minus_or_arrow(&mut ps),
			'*' => TokenKind::Asterisk,
			'/' => slash_or_comment(&mut ps)?,
			'=' => eq_or_fat_arrow_or_eqeq(&mut ps),
			'.' => TokenKind::Dot,
			',' => TokenKind::Comma,

			'>' => gt_or_gte(&mut ps),
			'<' => lt_or_lte(&mut ps),

			'(' => TokenKind::LeftParen,
			')' => TokenKind::RightParen,
			'{' => TokenKind::LeftBrace,
			'}' => TokenKind::RightBrace,
			'[' => TokenKind::LeftBracket,
			']' => TokenKind::RightBracket,

			'"' => string(&mut ps)?,

			c if c.is_ascii_alphabetic() || c == '_' => ident_or_keyword(&mut ps, c),
			c if c.is_ascii_digit() => number(&mut ps, c)?,

			c => {
				bail!(LexError::new(
					source.to_owned(),
					file.to_owned(),
					Position::new(start_index, start_index + 1),
					LexErrorKind::UnexpectedToken(errors::UnexpectedToken {
						got: c,
						expected: None
					})
				))
			}
		};

		let end_index = ps.peek().map_or(source.len(), |(idx, _)| *idx);
		let position = Position::new(start_index, end_index);
		let span = Span::new(source, position);

		ts.push(Token::new(kind, span));
	}

	ts.push(Token::new(
		TokenKind::Eof,
		Span::new(source, Position::new(source.len(), source.len()))
	));

	Ok(ts)
}

fn minus_or_arrow(ps: &mut ParseStream<'_>) -> TokenKind {
	if ps.peek_matches_exact('>', true) {
		TokenKind::Arrow
	} else {
		TokenKind::Minus
	}
}

fn slash_or_comment(ps: &mut ParseStream<'_>) -> Result<TokenKind> {
	let start_index = ps.position() - 1;

	if ps.peek_matches_exact('/', true) {
		// Single line comment
		ps.next_while_exact('\n', true);
		Ok(TokenKind::Comment)
	} else if ps.peek_matches_exact('*', true) {
		// Multiline comment
		if !ps.next_while_exact('*', true) {
			bail!(LexError::new(
				ps.source().to_owned(),
				ps.file().to_owned(),
				Position::new(start_index, ps.position()),
				LexErrorKind::UnexpectedToken(errors::UnexpectedToken {
					got: ' ',
					expected: Some('*')
				})
			));
		}

		if !ps.peek_matches_exact('/', true) {
			bail!(LexError::new(
				ps.source().to_owned(),
				ps.file().to_owned(),
				Position::new(start_index, ps.position()),
				LexErrorKind::UnexpectedToken(errors::UnexpectedToken {
					got: ' ',
					expected: Some('/')
				})
			));
		}

		Ok(TokenKind::Comment)
	} else {
		Ok(TokenKind::Slash)
	}
}

fn eq_or_fat_arrow_or_eqeq(ps: &mut ParseStream<'_>) -> TokenKind {
	if ps.peek_matches_exact('=', true) {
		TokenKind::EqEq
	} else if ps.peek_matches_exact('>', true) {
		TokenKind::FatArrow
	} else {
		TokenKind::Eq
	}
}

fn string(ps: &mut ParseStream<'_>) -> Result<TokenKind> {
	let start_index = ps.position() - 1;

	let mut closed = false;
	let mut last_char = (start_index, '"');

	while let Some((index, next_char)) = ps.peek() {
		last_char = (*index, *next_char);

		if *next_char == '\n' {
			break;
		}

		if *next_char == '"' {
			closed = true;
			ps.next();

			break;
		}

		ps.next();
	}

	if !closed {
		bail!(LexError::new(
			ps.source().to_owned(),
			ps.file().to_owned(),
			Position::new(last_char.0, last_char.0 + 1),
			LexErrorKind::UnexpectedToken(errors::UnexpectedToken {
				got: last_char.1,
				expected: Some('"')
			})
		))
	}

	Ok(TokenKind::String)
}

fn ident_or_keyword(ps: &mut ParseStream<'_>, c: char) -> TokenKind {
	let mut ident = String::new();
	ident.push(c);

	while let Some((_, next_char)) = ps.peek() {
		if !next_char.is_ascii_alphanumeric() && *next_char != '_' {
			break;
		}

		ident.push(*next_char);
		ps.next();
	}

	if let Some(kw) = Keyword::parse(&ident) {
		TokenKind::Keyword(kw)
	} else {
		TokenKind::Identifier
	}
}

fn number(ps: &mut ParseStream<'_>, c: char) -> Result<TokenKind> {
	let start_index = ps.position() - 1;

	let mut number = String::new();
	number.push(c);

	while let Some((_, next_char)) = ps.peek() {
		if !next_char.is_ascii_digit() {
			break;
		}

		number.push(*next_char);
		ps.next();
	}

	if number.parse::<f64>().is_err() {
		bail!(LexError::new(
			ps.source().to_owned(),
			ps.file().to_owned(),
			Position::new(start_index, ps.position()),
			LexErrorKind::MalformedNumber(errors::MalformedNumber)
		));
	}

	Ok(TokenKind::Number)
}

fn gt_or_gte(ps: &mut ParseStream<'_>) -> TokenKind {
	if ps.peek_matches_exact('=', true) {
		TokenKind::Gte
	} else {
		TokenKind::Gt
	}
}

fn lt_or_lte(ps: &mut ParseStream<'_>) -> TokenKind {
	if ps.peek_matches_exact('=', true) {
		TokenKind::Lte
	} else {
		TokenKind::Lt
	}
}
