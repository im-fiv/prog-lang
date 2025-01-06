use std::fmt::{self, Debug};
use std::marker::PhantomData;

use anyhow::Result;

use crate::{ASTNode, Parse, ParseStream, Position, Span};

#[derive(Clone, PartialEq)]
pub struct Punctuated<'inp, T, P> {
	pairs: Vec<(T, P)>,
	tail: Option<T>,
	_marker: PhantomData<(&'inp T, &'inp P)>
}

impl<'inp, T, P> Punctuated<'inp, T, P> {
	pub fn new() -> Self {
		Self {
			pairs: vec![],
			tail: None,
			_marker: PhantomData
		}
	}

	pub fn is_empty(&self) -> bool { self.pairs.is_empty() && self.tail.is_none() }

	pub fn len(&self) -> usize { self.pairs.len() + if self.tail.is_some() { 1 } else { 0 } }

	pub fn push_pair(&mut self, pair: (T, P)) { self.pairs.push(pair); }

	pub fn map<F, G, H, I>(self, f: F, g: G) -> Punctuated<'inp, H, I>
	where
		F: Fn(T) -> H,
		G: Fn(P) -> I
	{
		let pairs = self
			.pairs
			.into_iter()
			.map(|(t, p)| (f(t), g(p)))
			.collect::<Vec<_>>();

		let tail = self.tail.map(f);

		Punctuated {
			pairs,
			tail,
			_marker: PhantomData
		}
	}

	pub fn unwrap(self) -> (Vec<(T, P)>, Option<T>) { (self.pairs, self.tail) }

	pub fn unwrap_items(self) -> Vec<T> {
		let mut items = self
			.pairs
			.into_iter()
			.map(|(item, _)| item)
			.collect::<Vec<_>>();

		if let Some(t) = self.tail {
			items.push(t);
		}

		items
	}

	pub fn items(&self) -> Vec<&T> {
		let mut items = self.pairs.iter().map(|(item, _)| item).collect::<Vec<_>>();

		if let Some(ref t) = self.tail {
			items.push(t);
		}

		items
	}

	pub fn first(&self) -> Option<&T> { self.pairs.first().map(|(item, _)| item) }

	pub fn pop_first(&mut self) -> (T, P) { self.pairs.remove(0) }
}

impl<'inp> Punctuated<'inp, Position, Position> {
	pub fn position(&self) -> Position {
		assert!(
			!self.is_empty(),
			"Could not get punctuated list's position as it is empty"
		);

		match (!self.pairs.is_empty(), self.tail) {
			(true, Some(tail)) => {
				let end = tail.end();
				let start = self.pairs.first().map(|(item, _)| item.start()).unwrap();

				Position::new(start, end)
			}

			(false, Some(tail)) => tail,

			(true, None) => {
				let start = self.pairs.first().map(|(item, _)| item.start()).unwrap();

				let end = self.pairs.last().map(|(_, punct)| punct.start()).unwrap();

				Position::new(start, end)
			}

			(false, None) => unreachable!()
		}
	}
}

impl<'inp, T, P> ASTNode for Punctuated<'inp, T, P>
where
	T: Parse<'inp>,
	P: Parse<'inp>
{
	fn span(&self) -> Span {
		assert!(
			!self.is_empty(),
			"Could not get punctuated list's span as it is empty"
		);

		match (!self.pairs.is_empty(), self.tail.as_ref()) {
			(true, Some(tail)) => {
				let end = tail.end();
				let start = self.pairs.first().map(|(item, _)| item.start()).unwrap();

				let position = Position::new(start, end);
				Span::new(tail.source(), position)
			}

			(false, Some(tail)) => tail.span(),

			(true, None) => {
				let (start, source) = self
					.pairs
					.first()
					.map(|(item, _)| (item.start(), item.source()))
					.unwrap();

				let end = self.pairs.last().map(|(_, punct)| punct.start()).unwrap();

				let position = Position::new(start, end);
				Span::new(source, position)
			}

			(false, None) => unreachable!()
		}
	}
}

impl<'inp, T, P> Parse<'inp> for Punctuated<'inp, T, P>
where
	T: Parse<'inp>,
	P: Parse<'inp>
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

		for (t, p) in self.pairs.iter() {
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
