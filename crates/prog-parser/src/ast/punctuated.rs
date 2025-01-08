use std::fmt::{self, Debug};
use std::marker::PhantomData;

use crate::{errors, ASTNode, Parse, ParseError, ParseErrorKind, ParseResult, ParseStream, Position, Span, Token};

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

	pub fn push_item(&mut self, item: T) {
		assert!(
			self.tail.is_none(),
			"Unable to push item into a punctuated list as there is no punctuation behind it"
		);

		self.tail = Some(item);
	}

	pub fn push_punct(&mut self, punct: P) {
		let item = self.tail.take().expect(
			"Unable to push punctuation into a punctuated list as there is no item behind it"
		);

		self.push_pair((item, punct));
	}

	pub fn get_pair(&self, index: usize) -> Option<&(T, P)> { self.pairs.get(index) }

	pub fn remove_pair(&mut self, index: usize) -> Option<(T, P)> {
		if index >= self.pairs.len() {
			return None;
		}

		Some(self.pairs.remove(index))
	}

	pub fn remove_tail(&mut self) -> Option<T> { self.tail.take() }

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

	pub fn map_ref<F, G, H, I>(&self, f: F, g: G) -> Punctuated<'_, H, I>
	where
		F: Fn(&T) -> H,
		G: Fn(&P) -> I
	{
		let pairs = self
			.pairs
			.iter()
			.map(|(t, p)| (f(t), g(p)))
			.collect::<Vec<_>>();

		let tail = self.tail.as_ref().map(f);

		Punctuated {
			pairs,
			tail,
			_marker: PhantomData
		}
	}

	pub fn map_mut<F, G, H, I>(&mut self, f: F, g: G) -> Punctuated<'_, H, I>
	where
		F: Fn(&mut T) -> H,
		G: Fn(&mut P) -> I
	{
		let pairs = self
			.pairs
			.iter_mut()
			.map(|(t, p)| (f(t), g(p)))
			.collect::<Vec<_>>();

		let tail = self.tail.as_mut().map(f);

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

		if let Some(tail) = self.tail {
			items.push(tail);
		}

		items
	}

	pub fn unwrap_puncts(self) -> Vec<P> {
		self.pairs
			.into_iter()
			.map(|(_, punct)| punct)
			.collect::<Vec<_>>()
	}

	pub fn items(&self) -> Vec<&T> {
		let mut items = self.pairs.iter().map(|(item, _)| item).collect::<Vec<_>>();

		if let Some(ref tail) = self.tail {
			items.push(tail);
		}

		items
	}

	pub fn puncts(&self) -> Vec<&P> {
		self.pairs
			.iter()
			.map(|(_, punct)| punct)
			.collect::<Vec<_>>()
	}

	fn assert_non_empty(&self) { assert!(!self.is_empty(), "Punctuated list must not be empty") }
}

impl Punctuated<'_, Position, Position> {
	pub fn start(&self) -> usize {
		self.assert_non_empty();

		self.pairs
			.first()
			.map(|(item, _)| item.start())
			.or_else(|| self.tail.as_ref().map(|tail| tail.start()))
			.expect("`Punctuated::assert_non_empty` hasn't done its job")
	}

	pub fn end(&self) -> usize {
		self.assert_non_empty();

		self.tail
			.as_ref()
			.map(|tail| tail.end())
			.or_else(|| self.pairs.last().map(|(_, punct)| punct.end()))
			.expect("`Punctuated::assert_non_empty` hasn't done its job")
	}

	pub fn position(&self) -> Position { Position::new(self.start(), self.end()) }
}

impl<T, P> ASTNode for Punctuated<'_, T, P>
where
	T: ASTNode,
	P: ASTNode
{
	fn span(&self) -> Span {
		self.assert_non_empty();

		let pos_list = self.map_ref(T::position, P::position);

		let start = pos_list.start();
		let end = pos_list.end();

		let (source, file) = self
			.pairs
			.first()
			.map(|(item, _)| (item.source(), item.file()))
			.or_else(|| self.tail.as_ref().map(|tail| (tail.source(), tail.file())))
			.unwrap_or_else(|| unreachable!());
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'inp, T, P> Parse<'inp> for Punctuated<'inp, T, P>
where
	T: Parse<'inp>,
	P: Parse<'inp>
{
	fn parse(input: &ParseStream<'inp>) -> ParseResult<Self> {
		let mut list = Self::new();

		loop {
			let Ok(item) = input.try_parse::<T>() else {
				break;
			};

			let Ok(punct) = input.try_parse::<P>() else {
				list.tail = Some(item);
				break;
			};

			list.push_pair((item, punct));
		}

		if list.is_empty() {
			// TODO: refine this
			let token = input.peek().unwrap();
			let span = token.sp();

			return Err(ParseError::new(
				span.source().to_owned(),
				span.file().to_owned(),
				span.position(),
				ParseErrorKind::Internal(errors::Internal(
					String::from("Punctuated list did not parse any items")
				))
			));
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

		if let Some(ref t) = self.tail {
			s.field(t);
		}

		s.finish()
	}
}

impl<T, P> Default for Punctuated<'_, T, P> {
	fn default() -> Self { Self::new() }
}
