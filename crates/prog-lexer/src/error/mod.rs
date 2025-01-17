mod malformed_number;
mod unexpected_token;

pub use malformed_number::MalformedNumber;
pub use unexpected_token::UnexpectedToken;

use prog_utils::pretty_errors::{PrettyError, PrettyErrorKind};

pub type LexError<'s> = PrettyError<'s, LexErrorKind>;

#[derive(Debug, Clone, prog_macros::AriadneCompatible)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum LexErrorKind {
	UnexpectedToken(UnexpectedToken),
	MalformedNumber(MalformedNumber)
}

impl PrettyErrorKind<'_> for LexErrorKind {}
