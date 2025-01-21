mod error;
mod token;
mod stream;

pub use error::{LexError, LexErrorKind};
pub use stream::LexStream;
pub use token::{Token, TokenKind, TokenStream};

use prog_utils::pretty_errors::{Position, Span};

pub type LexResult<'s, T> = Result<T, LexError<'s>>;

fn unexpected_char<'src>(
	ls: &mut LexStream<'src>,
	found: char,
	expected: Option<char>,
	start: Option<usize>
) -> LexError<'src> {
	let position = Position::new(start.unwrap_or(ls.position() - 1), ls.position());

	LexError::from_raw_parts(
		ls.source(),
		ls.file(),
		position,
		LexErrorKind::UnexpectedChar(error::UnexpectedChar { found, expected })
	)
}

pub fn lex<'src>(source: &'src str, file: &'src str) -> LexResult<'src, TokenStream<'src>> {
	let mut ls = LexStream::new(source, file);
	let mut ts = TokenStream::new();

	while let Some((start_index, char)) = ls.next() {
		let kind = match char {
			' ' | '\t' | '\n' | '\r' => continue,

			'+' => TokenKind::Plus,
			'-' => minus_or_arrow(&mut ls),
			'*' => TokenKind::Asterisk,
			'/' => slash_or_comment(&mut ls)?,
			'%' => TokenKind::Sign,
			'=' => eq_or_fat_arrow_or_eqeq(&mut ls),
			'!' => neq(&mut ls)?,
			'.' => TokenKind::Dot,
			',' => TokenKind::Comma,

			'>' => gt_or_gte(&mut ls),
			'<' => lt_or_lte(&mut ls),

			'(' => TokenKind::LeftParen,
			')' => TokenKind::RightParen,
			'{' => TokenKind::LeftBrace,
			'}' => TokenKind::RightBrace,
			'[' => TokenKind::LeftBracket,
			']' => TokenKind::RightBracket,

			'"' => string(&mut ls)?,

			c if c.is_ascii_alphabetic() || c == '_' => ident_or_keyword(&mut ls, c),
			c if c.is_ascii_digit() => number(&mut ls, c)?,

			c => {
				return Err(LexError::from_raw_parts(
					source,
					file,
					Position::new(start_index, start_index + 1),
					LexErrorKind::UnexpectedChar(error::UnexpectedChar {
						found: c,
						expected: None
					})
				));
			}
		};

		let end_index = ls.peek().map_or(source.len(), |(idx, _)| *idx);
		let position = Position::new(start_index, end_index);
		let span = Span::new(source, file, position);

		ts.push(Token::new(kind, span));
	}

	ts.push(Token::new(
		TokenKind::Eof,
		Span::new(source, file, Position::new(source.len(), source.len()))
	));

	ts.filter_comments();

	Ok(ts)
}

fn minus_or_arrow(ls: &mut LexStream<'_>) -> TokenKind {
	if ls.peek_matches_exact('>', true) {
		TokenKind::Arrow
	} else {
		TokenKind::Minus
	}
}

fn slash_or_comment<'src>(ls: &mut LexStream<'src>) -> LexResult<'src, TokenKind> {
	let start_index = ls.position() - 1;

	if ls.peek_matches_exact('/', true) {
		// Single line comment
		ls.next_while_exact('\n', true);
		Ok(TokenKind::Comment)
	} else if ls.peek_matches_exact('*', true) {
		// Multiline comment
		if !ls.next_while_exact('*', true) {
			return Err(LexError::from_raw_parts(
				ls.source(),
				ls.file(),
				Position::new(start_index, ls.position()),
				LexErrorKind::UnexpectedChar(error::UnexpectedChar {
					found: ' ',
					expected: Some('*')
				})
			));
		}

		if !ls.peek_matches_exact('/', true) {
			return Err(LexError::from_raw_parts(
				ls.source(),
				ls.file(),
				Position::new(start_index, ls.position()),
				LexErrorKind::UnexpectedChar(error::UnexpectedChar {
					found: ' ',
					expected: Some('/')
				})
			));
		}

		Ok(TokenKind::Comment)
	} else {
		Ok(TokenKind::Slash)
	}
}

fn eq_or_fat_arrow_or_eqeq(ls: &mut LexStream<'_>) -> TokenKind {
	if ls.peek_matches_exact('=', true) {
		TokenKind::EqEq
	} else if ls.peek_matches_exact('>', true) {
		TokenKind::FatArrow
	} else {
		TokenKind::Eq
	}
}

fn neq<'src>(ls: &mut LexStream<'src>) -> LexResult<'src, TokenKind> {
	let start_index = ls.position() - 1;
	let next = ls.next();

	if next.is_none() {
		return Err(unexpected_char(ls, ' ', Some('='), Some(start_index)));
	}

	let next = next.unwrap();

	if next.1 != '=' {
		return Err(unexpected_char(ls, next.1, Some('='), Some(start_index)));
	}

	Ok(TokenKind::Neq)
}

fn string<'src>(ls: &mut LexStream<'src>) -> LexResult<'src, TokenKind> {
	let start_index = ls.position() - 1;

	let mut closed = false;
	let mut last_char = (start_index, '"');

	while let Some((index, next_char)) = ls.peek() {
		last_char = (*index, *next_char);

		if *next_char == '\n' {
			break;
		}

		if *next_char == '"' {
			closed = true;
			ls.next();

			break;
		}

		ls.next();
	}

	if !closed {
		return Err(LexError::from_raw_parts(
			ls.source(),
			ls.file(),
			Position::new(last_char.0, last_char.0 + 1),
			LexErrorKind::UnexpectedChar(error::UnexpectedChar {
				found: last_char.1,
				expected: Some('"')
			})
		));
	}

	Ok(TokenKind::String)
}

fn ident_or_keyword(ls: &mut LexStream<'_>, c: char) -> TokenKind {
	let mut ident = String::new();
	ident.push(c);

	while let Some((_, next_char)) = ls.peek() {
		if !next_char.is_ascii_alphanumeric() && *next_char != '_' {
			break;
		}

		ident.push(*next_char);
		ls.next();
	}

	if let Some(kw) = TokenKind::as_keyword(&ident) {
		kw
	} else {
		TokenKind::Ident
	}
}

// TODO: add exponents and hexadecimal/octal/binary formats
fn number<'src>(ls: &mut LexStream<'src>, c: char) -> LexResult<'src, TokenKind> {
	let start_index = ls.position() - 1;

	let mut number = String::new();
	number.push(c);

	let mut has_decimal = false;

	while let Some((_, char)) = ls.peek() {
		match char {
			'0'..='9' => {
				number.push(*char);
				ls.next();
			}

			'.' if !has_decimal => {
				has_decimal = true;
				number.push(*char);
				ls.next();
			}

			_ => break
		}
	}

	if number.parse::<f64>().is_err() {
		return Err(LexError::from_raw_parts(
			ls.source(),
			ls.file(),
			Position::new(start_index, ls.position()),
			LexErrorKind::MalformedNumber(error::MalformedNumber)
		));
	}

	Ok(TokenKind::Number)
}

fn gt_or_gte(ls: &mut LexStream<'_>) -> TokenKind {
	if ls.peek_matches_exact('=', true) {
		TokenKind::Gte
	} else {
		TokenKind::Gt
	}
}

fn lt_or_lte(ls: &mut LexStream<'_>) -> TokenKind {
	if ls.peek_matches_exact('=', true) {
		TokenKind::Lte
	} else {
		TokenKind::Lt
	}
}

#[cfg(test)]
mod tests {
	use TokenKind::*;

	use super::*;

	fn quick_lex(input: &str) -> Box<[TokenKind]> {
		let ts = lex(input, "<stdin").unwrap();
		let tokens = ts.unwrap();
		let kinds = tokens.into_iter().map(|t| t.kind()).collect::<Box<[_]>>();

		kinds
	}

	#[test]
	fn test_no_input() {
		assert_eq!(*quick_lex(""), [Eof]);
	}

	#[test]
	#[should_panic]
	fn test_invalid_token() { quick_lex("???"); }

	#[test]
	fn test_ident_vs_keyword() {
		assert_eq!(*quick_lex("hello world def true false not"), [
			Ident, Ident, Def, True, False, Not, Eof
		]);
	}

	#[test]
	fn test_numbers() {
		assert_eq!(*quick_lex("0 5 42 24"), [
			Number, Number, Number, Number, Eof
		]);
	}

	#[test]
	fn test_strings() {
		assert_eq!(*quick_lex("\"this is a string\" but this isnt"), [
			String, Ident, Ident, Ident, Eof
		]);
	}
}
