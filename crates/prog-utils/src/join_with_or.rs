use std::fmt::Display;

/// A trait for formatting a list of items into a human-readable string.
pub trait JoinWithOr {
	/// Type of items in the list.
	type Item: Display;

	/// Formats and joins the list into a human-readable string using the default formatting.
	///
	/// # Examples
	///
	/// ```
	/// # use prog_utils::JoinWithOr;
	/// # assert_eq!(<[i32] as JoinWithOr>::fmt_join(&[]), "");
	/// assert_eq!([1].fmt_join(), "1");
	/// assert_eq!([1, 2].fmt_join(), "1 or 2");
	/// assert_eq!([1, 2, 3].fmt_join(), "1, 2, or 3");
	/// assert_eq!([1, 2, 3, 4].fmt_join(), "1, 2, 3, or 4");
	/// ```
	fn fmt_join(&self) -> String { self.fmt_join_with(|item| format!("{item}")) }

	/// Formats and joins the list into a human-readable string using a custom formatting function.
	///
	/// # Examples
	///
	/// ```
	/// # use prog_utils::JoinWithOr;
	/// let as_hex = |num: &i32| format!("{num:#03X}");
	///
	/// assert_eq!([10].fmt_join_with(as_hex), "0xA");
	/// assert_eq!([10, 11].fmt_join_with(as_hex), "0xA or 0xB");
	/// assert_eq!([10, 11, 12].fmt_join_with(as_hex), "0xA, 0xB, or 0xC");
	/// assert_eq!([10, 11, 12, 13].fmt_join_with(as_hex), "0xA, 0xB, 0xC, or 0xD");
	/// ```
	fn fmt_join_with<F>(&self, format_item: F) -> String
	where
		F: Fn(&Self::Item) -> String;
}

impl<T: Display> JoinWithOr for [T] {
	type Item = T;

	fn fmt_join_with<F>(&self, format_item: F) -> String
	where
		F: Fn(&Self::Item) -> String
	{
		let mut fmt_items = self.iter().map(format_item).collect::<Vec<_>>();

		match fmt_items.len() {
			0 => String::new(),
			1 => fmt_items.remove(0),
			2 => format!("{} or {}", fmt_items[0], fmt_items[1]),

			_ => {
				let mut result = String::new();

				for (idx, item) in fmt_items.iter().enumerate() {
					result += item;

					match idx.cmp(&(fmt_items.len() - 2)) {
						std::cmp::Ordering::Less => result += ", ",
						std::cmp::Ordering::Equal => result += ", or ",
						_ => ()
					}
				}

				result
			}
		}
	}
}

impl<T: Display, const N: usize> JoinWithOr for [T; N]
where
	[T]: JoinWithOr
{
	type Item = <[T] as JoinWithOr>::Item;

	fn fmt_join_with<F>(&self, format_item: F) -> String
	where
		F: Fn(&Self::Item) -> String
	{
		self.as_slice().fmt_join_with(format_item)
	}
}
