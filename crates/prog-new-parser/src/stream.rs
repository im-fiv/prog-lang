use std::cell::RefCell;
use std::iter::Peekable;
use std::marker::PhantomData;
use std::vec::IntoIter;

use anyhow::Result;
use prog_utils::stream::{IteratorConvertion, Stream};

pub struct ParseStream<'inp> {
	iter: RefCell<<Self as Stream>::Iterator>,
	_lifetime: PhantomData<&'inp <Self as Stream>::Item>
}

impl<'inp> ParseStream<'inp> {
	pub fn new(input: Vec<<Self as Stream>::Item>) -> Self {
		let iter = input.into_iter().peekable();

		Self {
			iter: RefCell::new(iter),
			_lifetime: PhantomData
		}
	}

	pub fn parse<T: crate::Parse<'inp>>(&'inp self) -> Result<T> { T::parse(self) }

	pub fn expect(&self, kind: prog_lexer::TokenKind) -> Result<<Self as Stream>::Item> {
		let mut iter = self.iter.borrow_mut();
		let token = iter.next();

		if token.is_none() {
			// TODO
			todo!("error handling");
		}

		let token = token.unwrap();

		if token.kind() != kind {
			// TODO
			todo!("error handling")
		}

		Ok(token)
	}
}

impl IteratorConvertion<<Self as Stream>::Item> for ParseStream<'_> {
	type Output = <Self as Stream>::Item;

	fn convert_item(input: &<Self as Stream>::Item) -> &Self::Output { input }
}

impl<'inp> Stream for ParseStream<'inp> {
	type Item = prog_lexer::Token<'inp>;
	type Iterator = Peekable<IntoIter<Self::Item>>;

	fn next(&mut self) -> Option<Self::Item> { self.iter.get_mut().next() }

	fn peek(&mut self) -> Option<&Self::Item> { self.iter.get_mut().peek() }
}