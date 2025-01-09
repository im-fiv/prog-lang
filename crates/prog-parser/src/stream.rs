use std::cell::Cell;

use prog_lexer::{Token, TokenKind};

use crate::{error, ParseError, ParseErrorKind, ParseResult};

#[derive(Debug)]
pub struct ParseStream<'src> {
	buffer: &'src [Token<'src>],
	/// Current buffer index position.
	cursor: Cell<usize>
}

impl<'src> ParseStream<'src> {
	/// Creates a new `ParseStream` from a slice of tokens.
	pub fn new(buffer: &'src [Token<'src>]) -> Self {
		Self {
			buffer,
			cursor: Cell::new(0)
		}
	}
}

impl<'src> ParseStream<'src>
where
	Token<'src>: Copy
{
	/// Returns the length of the token buffer without taking the cursor into account.
	pub fn untracked_len(&self) -> usize { self.buffer.len() }

	/// Returns the *remaining* length of the token buffer with respect to the cursor position.
	pub fn len(&self) -> usize { self.untracked_len().saturating_sub(self.cursor()) }

	/// Returns whether the token buffer is empty without taking the cursor into account.
	pub fn untracked_is_empty(&self) -> bool { self.buffer.is_empty() }

	/// Returns whether the token buffer is empty with respect to the cursor position.
	pub fn is_empty(&self) -> bool { self.len() == 0 || self.untracked_is_empty() }

	/// Returns the current cursor position in the token buffer.
	pub fn cursor(&self) -> usize { self.cursor.get() }

	/// Sets the cursor position in the token buffer.
	pub(crate) fn set_cursor(&self, cursor: usize) {
		// Clamping the cursor such that it does not exceed the buffer length
		let cursor = cursor.min(self.untracked_len());
		self.cursor.set(cursor)
	}

	/// Parses a value of type `T` from the current position in the stream.
	///
	/// This function directly delegates the parsing to `T`'s `Parse` implementation.
	pub fn parse<T>(&'_ self) -> ParseResult<T>
	where
		T: crate::Parse<'src>
	{
		T::parse(self)
	}

	/// Attempts to parse a value of type `T` from the current position in the stream,
	/// but only *consumes* the tokens if the parse is successful.
	///
	/// This is useful for lookahead parsing where you might try different options
	/// without making *irreversible* progress in the stream.
	pub fn try_parse<T>(&'_ self) -> ParseResult<T>
	where
		T: crate::Parse<'src>
	{
		self.try_parse_with(Self::parse::<T>)
	}

	/// Attempts to parse a value of type `T` from the current position in the stream
	/// using the provided `parse` function.
	///
	/// The stream is forked before calling the `parse` function, and only if the
	/// parse is successful, the main stream cursor will be advanced, thus *consuming*
	/// the parsed tokens.
	pub fn try_parse_with<T, F>(&'_ self, parse: F) -> ParseResult<T>
	where
		T: crate::Parse<'src>,
		F: FnOnce(&Self) -> ParseResult<T>
	{
		let fork = self.fork();
		let result = parse(&fork);

		if result.is_ok() {
			self.set_cursor(fork.cursor());
		}

		result
	}

	/// Peeks at the current token in the stream without advancing the cursor.
	///
	/// Returns `Some(Token)` if a token is available, or `None` if the
	/// end of the buffer has been reached, or if there is no current token.
	pub fn current(&'_ self) -> Option<Token<'src>> {
		if self.is_empty() {
			return None;
		}

		let cursor = self.cursor.get().saturating_sub(1);
		self.buffer.get(cursor).copied()
	}

	/// Gets the next token from the stream and advances the cursor.
	///
	/// Returns `Some(Token)` if a token is available, or `None` if the
	/// end of the buffer has been reached.
	pub fn next(&'_ self) -> Option<Token<'src>> {
		if self.is_empty() {
			return None;
		}

		let cursor = self.cursor.get();
		let value = self.buffer.get(cursor).copied();

		self.cursor.set(cursor + 1);
		value
	}

	/// Retrieves the next token from the stream, advancing the cursor,
	/// or returns an error if no token is available (end of input).
	///
	/// This function combines the behavior of `next()` with the expectation
	/// that a token *should* exist and *be valid*.
	/// If there are no more tokens in the stream, it will return an `Err`, rather than just `None`.
	pub fn expect_next(&'_ self) -> ParseResult<Token<'src>> {
		// Utilizing `expect_peek` to reduce code repetition with error reporting
		let result = self.expect_peek();

		if result.is_ok() {
			let _ = self.next();
		}

		result
	}

	/// Peeks at the next token in the stream without advancing the cursor.
	///
	/// Returns `Some(Token)` if a token is available, or `None` if the
	/// end of the buffer has been reached.
	pub fn peek(&'_ self) -> Option<Token<'src>> {
		if self.is_empty() {
			return None;
		}

		let cursor = self.cursor.get();
		self.buffer
			.get(cursor)
			.and_then(|t| {
				match t.kind() {
					TokenKind::Eof => None,
					_ => Some(t)
				}
			})
			.copied()
	}

	/// Peeks at the next token from the stream without advancing the cursor,
	/// or returns an error if no token *valid* is available (end of input).
	///
	/// This function combines the behavior of `peek()` with a `Result`,
	/// which indicates if a *valid* token is available.
	pub fn expect_peek(&'_ self) -> ParseResult<Token<'src>> {
		let token = self
			.current()
			.ok_or(ParseError::new_unspanned(ParseErrorKind::Internal(
				error::Internal(String::from("unexpected end of input"))
			)))?;
		let span = token.span();

		self.peek().ok_or(ParseError::with_span(
			span,
			ParseErrorKind::UnexpectedToken(error::UnexpectedToken {
				got: TokenKind::Eof,
				expected: None
			})
		))
	}

	/// Peeks at the next token in the stream and returns it if its `TokenKind` matches the provided `kind`.
	///
	/// If no token is available, or the `TokenKind` does not match, `None` is returned.
	pub fn peek_matches(&'_ self, kind: TokenKind) -> Option<Token<'src>> {
		match self.peek() {
			Some(t) if t.kind() == kind => Some(t),
			_ => None
		}
	}

	/// Gets the next token from the stream and asserts that its `TokenKind`
	/// matches the given `kind`.
	///
	/// If the next token matches, it is returned wrapped in `Ok`.
	/// If the end of the stream is reached or token kinds do not match, an `Err` variant is returned instead.
	pub fn expect(&'_ self, kind: TokenKind) -> ParseResult<Token<'src>> {
		let token = self.expect_next()?;

		if token.kind() != kind {
			let span = token.span();

			return Err(ParseError::with_span(
				span,
				ParseErrorKind::UnexpectedToken(error::UnexpectedToken {
					got: token.kind(),
					expected: Some(kind)
				})
			));
		}

		Ok(token)
	}

	/// Creates a copy of the `ParseStream` with the same buffer, but a separate cursor.
	///
	/// This is used to implement backtracking by trying multiple alternative
	/// parsing branches.
	pub fn fork(&'_ self) -> Self {
		Self {
			buffer: self.buffer,
			cursor: self.cursor.clone()
		}
	}
}
