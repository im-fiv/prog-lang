mod unexpected_token;
mod malformed_number;

pub use malformed_number::*;
pub use unexpected_token::*;

use prog_utils::pretty_errors::{PrettyError, PrettyErrorKind};

pub type LexError = PrettyError<LexErrorKind>;

#[derive(Debug, Clone, prog_macros::AriadneCompatible)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum LexErrorKind {
	UnexpectedToken(UnexpectedToken),
	MalformedNumber(MalformedNumber)
}

impl PrettyErrorKind for LexErrorKind {}
