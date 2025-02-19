mod internal;
mod unexpected_token;

pub use internal::Internal;
pub use unexpected_token::UnexpectedToken;

use prog_utils::pretty_errors::{PrettyError, PrettyErrorKind};

pub type ParseError<'s> = PrettyError<'s, ParseErrorKind>;

#[derive(Debug, Clone, prog_macros::AriadneCompatible)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ParseErrorKind {
	Internal(Internal),
	UnexpectedToken(UnexpectedToken)
}

impl PrettyErrorKind<'_> for ParseErrorKind {}
