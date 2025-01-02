use std::iter::Peekable;
use std::str::CharIndices;

use prog_utils::stream::{IteratorConvertion, Stream};

#[derive(Debug)]
pub struct LexStream<'inp> {
	iter: <Self as Stream>::Iterator,
	source: &'inp str,
	file: &'inp str
}

impl<'inp> LexStream<'inp> {
	pub fn new(source: &'inp str, file: &'inp str) -> Self {
		let iter = source.char_indices().peekable();
		Self { iter, source, file }
	}

	pub fn source(&self) -> &'inp str { self.source }

	pub fn file(&self) -> &'inp str { self.file }

	pub fn position(&mut self) -> usize {
		let source_len = self.source.len();

		self.peek().map_or(source_len, |(idx, _)| *idx)
	}
}

impl IteratorConvertion<<Self as Stream>::Item> for LexStream<'_> {
	type Output = char;

	fn convert_item(input: &<Self as Stream>::Item) -> &Self::Output { &input.1 }
}

impl<'inp> Stream for LexStream<'inp> {
	type Item = <CharIndices<'inp> as Iterator>::Item;
	type Iterator = Peekable<CharIndices<'inp>>;

	fn next(&mut self) -> Option<Self::Item> { self.iter.next() }

	fn peek(&mut self) -> Option<&Self::Item> { self.iter.peek() }
}
