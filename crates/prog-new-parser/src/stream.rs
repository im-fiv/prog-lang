use std::cell::Cell;

use anyhow::{bail, Result};

#[derive(Debug)]
pub struct ParseStream<'inp> {
	buffer: &'inp [prog_lexer::Token<'inp>],
	/// Current buffer index position.
	cursor: Cell<usize>
}

impl<'inp> ParseStream<'inp> {
	/// Creates a new `ParseStream` from a slice of tokens.
	pub fn new(buffer: &'inp [prog_lexer::Token<'inp>]) -> Self {
		Self {
			buffer,
			cursor: Cell::new(0)
		}
	}
}

impl<'inp> ParseStream<'inp>
where
	prog_lexer::Token<'static>: Copy
{
	/// Returns the current cursor position in the token buffer.
	pub fn cursor(&self) -> usize { self.cursor.get() }

	/// Sets the cursor position in the token buffer.
	pub(crate) fn set_cursor(&self, cursor: usize) { self.cursor.set(cursor) }

	/// Parses a value of type `T` from the current position in the stream.
	///
	/// This function directly delegates the parsing to `T`'s `Parse` implementation.
	pub fn parse<T: crate::Parse<'inp>>(&'_ self) -> Result<T> { T::parse(self) }

	/// Attempts to parse a value of type `T` from the current position in the stream,
	/// but only *consumes* the tokens if the parse is successful.
	///
	/// This is useful for lookahead parsing where you might try different options
	/// without making *irreversible* progress in the stream.
	pub fn try_parse<T: crate::Parse<'inp>>(&'_ self) -> Result<T> {
		Self::try_parse_with(self, |input| input.parse::<T>())
	}

	/// Attempts to parse a value of type `T` from the current position in the stream
	/// using the provided `parse` function.
	///
	/// The stream is forked before calling the `parse` function, and only if the
	/// parse is successful, the main stream cursor will be advanced, thus *consuming*
	/// the parsed tokens.
	pub fn try_parse_with<T, F>(&'_ self, parse: F) -> Result<T>
	where
		T: crate::Parse<'inp>,
		F: FnOnce(&Self) -> Result<T>
	{
		let fork = self.fork();
		let result = parse(&fork);

		if result.is_ok() {
			self.set_cursor(fork.cursor());
		}

		result
	}

	/// Gets the next token from the stream and advances the cursor.
	///
	/// Returns `Some(Token)` if a token is available, or `None` if the
	/// end of the buffer has been reached.
	pub fn next(&'_ self) -> Option<prog_lexer::Token<'inp>> {
		if self.cursor.get() >= self.buffer.len() {
			return None;
		}

		let cursor = self.cursor.get();
		let value = self.buffer.get(cursor).copied();

		self.cursor.set(cursor + 1);
		value
	}

	/// Peeks at the next token in the stream without advancing the cursor.
	///
	/// Returns `Some(Token)` if a token is available, or `None` if the
	/// end of the buffer has been reached.
	pub fn peek(&'_ self) -> Option<prog_lexer::Token<'inp>> {
		if self.cursor.get() >= self.buffer.len() {
			return None;
		}

		let cursor = self.cursor.get();
		self.buffer.get(cursor).copied()
	}

	/// Peeks at the next token in the stream and returns it if its `TokenKind` matches the provided `kind`.
	///
	/// If no token is available, or the `TokenKind` does not match, `None` is returned.
	pub fn peek_matches(&'_ self, kind: prog_lexer::TokenKind) -> Option<prog_lexer::Token<'inp>> {
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
	pub fn expect(&'_ self, kind: prog_lexer::TokenKind) -> Result<prog_lexer::Token<'inp>> {
		let token = self.next();

		if token.is_none() {
			// TODO: error handling
			bail!("Unexpected EOI");
		}

		let token = token.unwrap();

		if token.kind() != kind {
			// TODO: error handling
			bail!(
				"Token kind mismatch (got={:?} != expected:{:?})",
				token.kind(),
				kind
			);
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
