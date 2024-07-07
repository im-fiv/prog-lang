mod expected_rules;
pub use expected_rules::ExpectedRules;
use prog_macros::ImplAriadneCompatible;
use prog_utils::pretty_errors::{PrettyError, PrettyErrorKind};

pub type ParseError = PrettyError<ParseErrorKind>;

#[derive(Debug, Clone, ImplAriadneCompatible)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub enum ParseErrorKind {
	ExpectedRules(ExpectedRules)
}

impl PrettyErrorKind for ParseErrorKind {}
