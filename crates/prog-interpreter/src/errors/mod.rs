mod unsupported_statement;
pub use unsupported_statement::*;

use serde::Serialize;

use prog_utils::pretty_errors::PrettyError;
use prog_macros::ImplAriadneCompatible;

pub type InterpretError = PrettyError<InterpretErrorKind>;

#[derive(Debug, Clone, Serialize, ImplAriadneCompatible)]
pub enum InterpretErrorKind {
	UnsupportedStatement(UnsupportedStatement)
}