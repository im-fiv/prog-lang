pub mod expressions;

mod misc;
mod statements;

pub use expressions::Expression;
pub use statements::*;
pub use misc::*;

pub use prog_utils::pretty_errors::Position;

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

pub(crate) use impl_basic_conv;