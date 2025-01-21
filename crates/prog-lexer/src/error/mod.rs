mod malformed_number;
mod unexpected_char;

pub use malformed_number::MalformedNumber;
pub use unexpected_char::UnexpectedChar;

use prog_utils::pretty_errors::{PrettyError, PrettyErrorKind};

pub type LexError<'s> = PrettyError<'s, LexErrorKind>;

#[derive(Debug, Clone, prog_macros::AriadneCompatible)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum LexErrorKind {
	MalformedNumber(MalformedNumber),
	UnexpectedChar(UnexpectedChar)
}

impl PrettyErrorKind<'_> for LexErrorKind {}
