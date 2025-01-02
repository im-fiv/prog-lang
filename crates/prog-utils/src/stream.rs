pub trait IteratorConvertion<Input> {
	type Output;

	fn convert_item(input: &Input) -> &Self::Output;
}

pub trait Stream: IteratorConvertion<Self::Item> {
	type Item;
	type Iterator: Iterator<Item = Self::Item>;

	/// Advances the iterator and returns the next value.
	fn next(&mut self) -> Option<Self::Item>;

	/// Returns a reference to the next() value without advancing the iterator.
	fn peek(&mut self) -> Option<&Self::Item>;

	/// Checks whether the next value matches a predicate without advancing the iterator.
	fn peek_matches<F>(&mut self, pred: F, consume: bool) -> bool
	where
		F: FnOnce(&<Self as IteratorConvertion<Self::Item>>::Output) -> bool
	{
		let matches = self.peek().map_or(false, |input| {
			let output = <Self as IteratorConvertion<Self::Item>>::convert_item(input);

			pred(output)
		});

		if matches && consume {
			self.next();
		}

		matches
	}

	/// Checks whether the next value matches an exact value without advancing the iterator.
	fn peek_matches_exact(
		&mut self,
		pred: <Self as IteratorConvertion<Self::Item>>::Output,
		consume: bool
	) -> bool
	where
		<Self as IteratorConvertion<Self::Item>>::Output: PartialEq,
		Self::Item: PartialEq
	{
		self.peek_matches(|c| *c == pred, consume)
	}

	/// Advances the iterator until the predicate returns `false`.
	///
	/// Return value of `true` indicates that a match was found before the iterator was exhausted.
	fn next_while<F>(&mut self, pred: F) -> bool
	where
		F: Fn(&Self::Item) -> bool
	{
		while let Some(item) = self.peek() {
			if !pred(item) {
				return true;
			}

			self.next();
		}

		false
	}

	/// Advances the iterator until the value exactly matches the predicate.
	///
	/// Return value of `true` indicates that a match was found before the iterator was exhausted.
	fn next_while_exact(
		&mut self,
		pred: <Self as IteratorConvertion<Self::Item>>::Output,
		consume: bool
	) -> bool
	where
		<Self as IteratorConvertion<Self::Item>>::Output: PartialEq
	{
		let matches = self.next_while(|input| {
			let output = <Self as IteratorConvertion<Self::Item>>::convert_item(input);

			output != &pred
		});

		if matches && consume {
			self.next();
		}

		matches
	}
}