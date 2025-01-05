use std::fmt::{self, Debug};
use std::marker::PhantomData;

use anyhow::Result;

use crate::{ASTNode, Parse, ParseStream, Position, Span, Token};

#[derive(Clone, PartialEq)]
pub struct Punctuated<'inp, T, P> {
	pub items: Vec<(T, P)>,
	pub tail: Option<T>,
	pub _marker: PhantomData<&'inp (T, P)>
}

impl<'inp, T, P> Punctuated<'inp, T, P> {
	pub fn new() -> Self {
		Self {
			items: vec![],
			tail: None,
			_marker: PhantomData
		}
	}

	pub fn is_empty(&self) -> bool { self.items.is_empty() && self.tail.is_none() }

	pub fn len(&self) -> usize { self.items.len() + if self.tail.is_some() { 1 } else { 0 } }

	pub fn push_pair(&mut self, pair: (T, P)) { self.items.push(pair); }
}

impl<'inp, T, P> ASTNode<'inp> for Punctuated<'inp, T, P>
where
	T: Parse<'inp>,
	P: Parse<'inp> + Token<'inp>
{
	fn span(&'inp self) -> Span {
		assert!(
			!self.is_empty(),
			"Could not get punctuated list's span as it is empty"
		);

		let (source, start) = self
			.items
			.first()
			.map(|(item, _)| (item.span().source(), item.span().start()))
			.unwrap();

		let end = match self.tail {
			Some(ref tail) => tail.span().end(),

			None => {
				self.items
					.last()
					.map(|(_, punct)| Token::span(punct).end())
					.unwrap()
			}
		};

		let position = Position::new(start, end);
		Span::new(source, position)
	}
}

impl<'inp, T, P> Parse<'inp> for Punctuated<'inp, T, P>
where
	T: Parse<'inp>,
	P: Parse<'inp> + Token<'inp>
{
	fn parse(input: &ParseStream<'inp>) -> Result<Self> {
		let mut list = Self::new();

		loop {
			let item = input.try_parse::<T>();
			if item.is_err() {
				break;
			}

			let punct = input.try_parse::<P>();
			if punct.is_err() {
				list.tail = Some(item?);
				break;
			}

			list.push_pair((item?, punct?));
		}

		Ok(list)
	}
}

impl<T, P> Debug for Punctuated<'_, T, P>
where
	T: Debug,
	P: Debug
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut s = f.debug_tuple("Punctuated");

		for (t, p) in self.items.iter() {
			s.field(t);
			s.field(p);
		}

		if let Some(t) = self.tail.as_ref() {
			s.field(t);
		}

		s.finish()
	}
}

impl<T, P> Default for Punctuated<'_, T, P>
where
	T: Clone,
	P: Clone
{
	fn default() -> Self { Self::new() }
}
