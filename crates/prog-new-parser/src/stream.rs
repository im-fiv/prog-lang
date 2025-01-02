use std::iter::Peekable;
use std::marker::PhantomData;
use std::vec::IntoIter;

use anyhow::Result;
use prog_utils::stream::{IteratorConvertion, Stream};

pub struct ParseStream<'inp> {
	iter: <Self as Stream>::Iterator,
	_lifetime: PhantomData<&'inp <Self as Stream>::Item>
}

impl ParseStream<'_> {
	pub fn expect(&self, _kind: prog_lexer::TokenKind) -> Result<()> { todo!() }
}

impl<'inp> IteratorConvertion<<Self as Stream>::Item> for ParseStream<'inp> {
	type Output = <Self as Stream>::Item;

	fn convert_item(input: &<Self as Stream>::Item) -> &Self::Output { input }
}

impl<'inp> Stream for ParseStream<'inp> {
	type Item = prog_lexer::Token<'inp>;
	type Iterator = Peekable<IntoIter<Self::Item>>;

	fn next(&mut self) -> Option<Self::Item> { self.iter.next() }

	fn peek(&mut self) -> Option<&Self::Item> { self.iter.peek() }
}
