mod join_with_or;
pub mod pretty_errors;

pub use join_with_or::JoinWithOr;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

pub fn read_file<P: AsRef<Path>>(path: P) -> String {
	let displayable = path.as_ref().display();

	let file = File::open(&path)
		.unwrap_or_else(|_| panic!("Failed to open file `{}` (read)", displayable));

	let mut reader = BufReader::new(file);
	let mut contents = String::new();

	reader
		.read_to_string(&mut contents)
		.unwrap_or_else(|_| panic!("Failed to read from file `{}`", displayable));

	contents.replace("\r\n", "\n")
}

#[macro_export]
macro_rules! impl_basic_conv {
	(from $from:ty => $for:ty as $variant:ident $({ $preproc:path })?) => {
		impl From<$from> for $for {
			fn from(value: $from) -> Self {
				Self::$variant(
					$( $preproc )? (value)
				)
			}
		}
	};
}
