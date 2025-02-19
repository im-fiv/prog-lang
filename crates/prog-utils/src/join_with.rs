use std::fmt::Display;

/// A trait for formatting a list of items into a human-readable string.
pub trait JoinWith {
	/// Type of items in the list.
	type Item: Display;

	/// Formats and joins the list into a human-readable string using the default formatting.
	///
	/// # Examples
	///
	/// ```
	/// # use prog_utils::JoinWith;
	/// # assert_eq!(<[i32] as JoinWith>::fmt_join(&[], ""), "");
	/// let tail = "or";
	/// assert_eq!([1].fmt_join(tail), "1");
	/// assert_eq!([1, 2].fmt_join(tail), "1 or 2");
	/// assert_eq!([1, 2, 3].fmt_join(tail), "1, 2, or 3");
	/// assert_eq!([1, 2, 3, 4].fmt_join(tail), "1, 2, 3, or 4");
	///
	/// let tail = "and";
	/// assert_eq!([1].fmt_join(tail), "1");
	/// assert_eq!([1, 2].fmt_join(tail), "1 and 2");
	/// assert_eq!([1, 2, 3].fmt_join(tail), "1, 2, and 3");
	/// assert_eq!([1, 2, 3, 4].fmt_join(tail), "1, 2, 3, and 4");
	/// ```
	fn fmt_join<T>(&self, tail: T) -> String
	where
		T: Display
	{
		self.fmt_join_with(|item| format!("{item}"), tail)
	}

	/// Formats and joins the list into a human-readable string using a custom formatting function.
	///
	/// # Examples
	///
	/// ```
	/// # use prog_utils::JoinWith;
	/// let as_hex = |num: &i32| format!("{num:#03X}");
	/// let tail = "or";
	///
	/// assert_eq!([10].fmt_join_with(as_hex, tail), "0xA");
	/// assert_eq!([10, 11].fmt_join_with(as_hex, tail), "0xA or 0xB");
	/// assert_eq!([10, 11, 12].fmt_join_with(as_hex, tail), "0xA, 0xB, or 0xC");
	/// assert_eq!([10, 11, 12, 13].fmt_join_with(as_hex, tail), "0xA, 0xB, 0xC, or 0xD");
	/// ```
	fn fmt_join_with<F, T>(&self, format_item: F, tail: T) -> String
	where
		F: Fn(&Self::Item) -> String,
		T: Display;
}

impl<I: Display> JoinWith for [I] {
	type Item = I;

	fn fmt_join_with<F, T>(&self, format_item: F, tail: T) -> String
	where
		F: Fn(&Self::Item) -> String,
		T: Display
	{
		let mut fmt_items = self.iter().map(format_item).collect::<Vec<_>>();

		match fmt_items.len() {
			0 => String::new(),
			1 => fmt_items.remove(0),
			2 => format!("{} {tail} {}", fmt_items[0], fmt_items[1]),

			_ => {
				let mut result = String::new();

				for (idx, item) in fmt_items.iter().enumerate() {
					result += item;

					match idx.cmp(&(fmt_items.len() - 2)) {
						std::cmp::Ordering::Less => result += ", ",
						std::cmp::Ordering::Equal => result += &format!(", {tail} "),
						_ => ()
					}
				}

				result
			}
		}
	}
}

impl<I: Display, const N: usize> JoinWith for [I; N]
where
	[I]: JoinWith
{
	type Item = <[I] as JoinWith>::Item;

	fn fmt_join_with<F, T>(&self, format_item: F, tail: T) -> String
	where
		F: Fn(&Self::Item) -> String,
		T: Display
	{
		self.as_slice().fmt_join_with(format_item, tail)
	}
}
