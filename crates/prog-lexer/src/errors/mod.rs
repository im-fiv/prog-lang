mod unexpected_token;
mod malformed_number;

pub use malformed_number::*;
use prog_macros::ImplAriadneCompatible;
use prog_utils::pretty_errors::{PrettyError, PrettyErrorKind};
pub use unexpected_token::*;

pub type LexError = PrettyError<LexErrorKind>;

#[derive(Debug, Clone, ImplAriadneCompatible)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum LexErrorKind {
	UnexpectedToken(UnexpectedToken),
	MalformedNumber(MalformedNumber)
}

impl PrettyErrorKind for LexErrorKind {}
