use std::cell::RefCell;
use std::iter::Peekable;
use std::vec::IntoIter;

use anyhow::Result;

pub struct ParseStream<'inp> {
	iter: RefCell<Peekable<IntoIter<prog_lexer::Token<'inp>>>>
}

impl<'inp> ParseStream<'inp> {
	pub fn new(input: Vec<prog_lexer::Token<'inp>>) -> Self {
		let iter = input.into_iter().peekable();

		Self {
			iter: RefCell::new(iter)
		}
	}

	pub fn parse<T: crate::Parse<'inp>>(&'inp self) -> Result<T> { T::parse(self) }

	pub fn next(&self) -> Option<prog_lexer::Token<'inp>> { self.iter.borrow_mut().next() }

	pub fn peek(&self) -> Option<prog_lexer::Token<'inp>>
	where
		prog_lexer::Token<'inp>: Copy
	{
		let mut iter = self.iter.borrow_mut();
		let item = iter.peek();

		item.copied()
	}

	pub fn expect(&self, kind: prog_lexer::TokenKind) -> Result<prog_lexer::Token<'inp>> {
		let mut iter = self.iter.borrow_mut();
		let token = iter.next();

		if token.is_none() {
			// TODO: error handling
			todo!("error handling");
		}

		let token = token.unwrap();

		if token.kind() != kind {
			// TODO: error handling
			todo!("error handling")
		}

		Ok(token)
	}
}
