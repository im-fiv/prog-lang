mod formatter;
pub use formatter::Formatter;

use std::fmt::Result;

mod __private {
	pub struct Private;
}

/// *Extension* of [`Display`] and [`Debug`] for values which require multiple formatting variants.
pub trait Printable {
	/// Writes to a string using a specified formatting function.
	///
	/// This function is not associative such that the trait is dyn-compliant.
	fn fmt_to_string<F>(&self, func: F, _: __private::Private) -> String
	where
		F: FnOnce(&Self, &mut Formatter) -> Result
	{
		let mut output = String::new();
		let mut formatter = Formatter::new(&mut output);
		func(self, &mut formatter).unwrap();

		output
	}

	/// Writes a concise, partial representation of `self` to the formatter `f`.
	///
	/// This format is intended for situations where a short, human-readable string is needed,
	/// such as logs, debugging, or a basic overview of the value.
	fn fmt_short(&self, f: &mut Formatter) -> Result;

	/// Unlike [`Printable::fmt_short`], writes the resulting representation
	/// and returns it as a `String`.
	fn str_short(&self) -> String { self.fmt_to_string(Self::fmt_short, __private::Private) }

	/// Writes a pretty-printed and relatively verbose representation of `self` to the formatter `f`,
	/// similar to the alternate formatting of [`Display`] and [`Debug`].
	///
	/// This is intended for situations where the user needs a more complete view of the value.
	/// The formatting will typically include line breaks and indentation in favor of readability.
	fn fmt_pretty(&self, f: &mut Formatter) -> Result;

	/// Unlike [`Printable::fmt_pretty`], writes the resulting representation
	/// and returns it as a `String`.
	fn str_pretty(&self) -> String { self.fmt_to_string(Self::fmt_pretty, __private::Private) }

	/// Writes a full, detailed representation of `self` to the formatter `f`,
	/// designed to distinguish even the most similar values,
	/// similar to a non-partial formatting achieved with [`Debug`].
	///
	/// This format is intended for debugging purposes or other cases
	/// where the complete state of the value needs to be inspected.
	fn fmt_full(&self, f: &mut Formatter) -> Result;

	/// Unlike [`Printable::fmt_full`], writes the resulting representation
	/// and returns it as a `String`.
	fn str_full(&self) -> String { self.fmt_to_string(Self::fmt_full, __private::Private) }
}
