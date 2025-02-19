use std::fmt::{self, Debug};
use std::marker::PhantomData;

use crate::{
	error, ASTNode, Parse, ParseError, ParseErrorKind, ParseResult, ParseStream, Position, Span
};

#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Punctuated<'src, Item, Punct> {
	pairs: Vec<(Item, Punct)>,
	tail: Option<Item>,

	_lt_item: PhantomData<&'src ()>,
	_lt_punct: PhantomData<&'src ()>
}

impl<'src, Item, Punct> Punctuated<'src, Item, Punct> {
	pub fn new() -> Self {
		Self {
			pairs: vec![],
			tail: None,

			_lt_item: PhantomData,
			_lt_punct: PhantomData
		}
	}

	pub fn is_empty(&self) -> bool { self.pairs.is_empty() && self.tail.is_none() }

	pub fn len(&self) -> usize { self.pairs.len() + if self.tail.is_some() { 1 } else { 0 } }

	pub fn push_pair(&mut self, pair: (Item, Punct)) { self.pairs.push(pair); }

	pub fn push_item(&mut self, item: Item) {
		assert!(
			self.tail.is_none(),
			"Unable to push item into a punctuated list as there is no punctuation behind it"
		);

		self.tail = Some(item);
	}

	pub fn push_punct(&mut self, punct: Punct) {
		let item = self.tail.take().expect(
			"Unable to push punctuation into a punctuated list as there is no item behind it"
		);

		self.push_pair((item, punct));
	}

	pub fn get_pair(&self, index: usize) -> Option<&(Item, Punct)> { self.pairs.get(index) }

	pub fn remove_pair(&mut self, index: usize) -> Option<(Item, Punct)> {
		if index >= self.pairs.len() {
			return None;
		}

		Some(self.pairs.remove(index))
	}

	pub fn remove_tail(&mut self) -> Option<Item> { self.tail.take() }

	//* `map` variants *//
	pub fn map<ItemPred, PunctPred, NewItem, NewPunct>(
		self,
		item_pred: ItemPred,
		punct_pred: PunctPred
	) -> Punctuated<'src, NewItem, NewPunct>
	where
		ItemPred: Fn(Item) -> NewItem,
		PunctPred: Fn(Punct) -> NewPunct
	{
		let pairs = self
			.pairs
			.into_iter()
			.map(|(i, p)| (item_pred(i), punct_pred(p)))
			.collect::<Vec<_>>();

		let tail = self.tail.map(item_pred);

		Punctuated {
			pairs,
			tail,

			_lt_item: PhantomData,
			_lt_punct: PhantomData
		}
	}

	pub fn map_items<ItemPred, NewItem>(
		self,
		item_pred: ItemPred
	) -> Punctuated<'src, NewItem, Punct>
	where
		ItemPred: Fn(Item) -> NewItem
	{
		self.map(item_pred, |p| p)
	}

	pub fn map_puncts<PunctPred, NewPunct>(
		self,
		punct_pred: PunctPred
	) -> Punctuated<'src, Item, NewPunct>
	where
		PunctPred: Fn(Punct) -> NewPunct
	{
		self.map(|i| i, punct_pred)
	}

	//* `map_ref` variants *//
	pub fn map_ref<ItemPred, PunctPred, NewItem, NewPunct>(
		&self,
		item_pred: ItemPred,
		punct_pred: PunctPred
	) -> Punctuated<'src, NewItem, NewPunct>
	where
		ItemPred: Fn(&Item) -> NewItem,
		PunctPred: Fn(&Punct) -> NewPunct
	{
		let pairs = self
			.pairs
			.iter()
			.map(|(i, p)| (item_pred(i), punct_pred(p)))
			.collect::<Vec<_>>();

		let tail = self.tail.as_ref().map(item_pred);

		Punctuated {
			pairs,
			tail,

			_lt_item: PhantomData,
			_lt_punct: PhantomData
		}
	}

	pub fn map_ref_items<ItemPred, NewItem>(
		&self,
		item_pred: ItemPred
	) -> Punctuated<'src, NewItem, Punct>
	where
		ItemPred: Fn(&Item) -> NewItem,
		Punct: Clone
	{
		self.map_ref(item_pred, |p| p.clone())
	}

	pub fn map_ref_puncts<PunctPred, NewPunct>(
		&self,
		punct_pred: PunctPred
	) -> Punctuated<'src, Item, NewPunct>
	where
		Item: Clone,
		PunctPred: Fn(&Punct) -> NewPunct
	{
		self.map_ref(|i| i.clone(), punct_pred)
	}

	// NOTE: `map_mut` implementation is convoluted, add when needed

	pub fn unwrap(self) -> (Vec<(Item, Punct)>, Option<Item>) { (self.pairs, self.tail) }

	pub fn unwrap_items(self) -> Vec<Item> {
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

	pub fn unwrap_puncts(self) -> Vec<Punct> {
		self.pairs
			.into_iter()
			.map(|(_, punct)| punct)
			.collect::<Vec<_>>()
	}

	pub fn items(&self) -> Vec<&Item> {
		let mut items = self.pairs.iter().map(|(item, _)| item).collect::<Vec<_>>();

		if let Some(ref tail) = self.tail {
			items.push(tail);
		}

		items
	}

	pub fn puncts(&self) -> Vec<&Punct> {
		self.pairs
			.iter()
			.map(|(_, punct)| punct)
			.collect::<Vec<_>>()
	}

	pub fn nth_item(&self, index: usize) -> Option<&Item> {
		if index >= self.len() {
			return None;
		}

		if index >= self.pairs.len() {
			return self.tail.as_ref();
		}

		self.pairs.get(index).map(|(i, _)| i)
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

impl<'src> ASTNode<'src> for Punctuated<'src, Span<'src>, Span<'src>> {
	fn span<'a>(&'a self) -> Span<'src> {
		self.assert_non_empty();

		let pos_list = self.map_ref(Span::position, Span::position);

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

impl<'src, Item, Punct> ASTNode<'src> for Punctuated<'src, Item, Punct>
where
	Item: ASTNode<'src>,
	Punct: ASTNode<'src>
{
	fn span<'a>(&'a self) -> Span<'src> {
		self.assert_non_empty();
		self.map_ref(ASTNode::span, ASTNode::span).span()
	}
}

impl<'src, Item, Punct> Parse<'src> for Punctuated<'src, Item, Punct>
where
	Item: Parse<'src>,
	Punct: Parse<'src>
{
	fn parse(input: &ParseStream<'src, '_>) -> ParseResult<'src, Self> {
		let mut list = Self::new();

		loop {
			let Ok(item) = input.try_parse::<Item>() else {
				break;
			};

			let Ok(punct) = input.try_parse::<Punct>() else {
				list.tail = Some(item);
				break;
			};

			list.push_pair((item, punct));
		}

		if list.is_empty() {
			return Err(ParseError::new_unspanned(ParseErrorKind::Internal(
				error::Internal(String::from("punctuated list did not parse any items"))
			)));
		}

		Ok(list)
	}
}

impl<Item, Punct> Debug for Punctuated<'_, Item, Punct>
where
	Item: Debug,
	Punct: Debug
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

impl<Item, Punct> Default for Punctuated<'_, Item, Punct> {
	fn default() -> Self { Self::new() }
}
