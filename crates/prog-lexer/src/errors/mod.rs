mod unexpected_token;
pub use unexpected_token::*;

use prog_macros::ImplAriadneCompatible;
use prog_utils::pretty_errors::{PrettyError, PrettyErrorKind};

pub type LexError = PrettyError<LexErrorKind>;

#[derive(Debug, Clone, ImplAriadneCompatible)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum LexErrorKind {
	UnexpectedToken(UnexpectedToken)
}

impl PrettyErrorKind for LexErrorKind {}