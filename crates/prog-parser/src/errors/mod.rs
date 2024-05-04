mod expected_rules;
pub use expected_rules::ExpectedRules;

use serde::Serialize;

use prog_utils::pretty_errors::PrettyError;
use prog_macros::ImplAriadneCompatible;

pub type ParseError = PrettyError<ParseErrorKind>;

#[derive(Debug, Clone, Serialize, ImplAriadneCompatible)]
pub enum ParseErrorKind {
	ExpectedRules(ExpectedRules)
}