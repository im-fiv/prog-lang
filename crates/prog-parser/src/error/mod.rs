mod internal;

pub use internal::Internal;

use prog_macros::ImplAriadneCompatible;
use prog_utils::pretty_errors::{PrettyError, PrettyErrorKind};

pub type ParseError = PrettyError<ParseErrorKind>;

#[derive(Debug, Clone, ImplAriadneCompatible)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ParseErrorKind {
	Internal(Internal)
}

impl PrettyErrorKind for ParseErrorKind {}
