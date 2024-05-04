mod argument_count_mismatch;
mod argument_type_mismatch;
mod context_disallowed;
mod unsupported_statement;
mod value_already_exists;
mod value_doesnt_exist;

pub use argument_count_mismatch::*;
pub use argument_type_mismatch::*;
pub use context_disallowed::*;
pub use unsupported_statement::*;
pub use value_already_exists::*;
pub use value_doesnt_exist::*;

use serde::Serialize;

use prog_utils::pretty_errors::PrettyError;
use prog_macros::ImplAriadneCompatible;

pub type InterpretError = PrettyError<InterpretErrorKind>;

#[derive(Debug, Clone, Serialize, ImplAriadneCompatible)]
pub enum InterpretErrorKind {
	ArgumentCountMismatch(ArgumentCountMismatch),
	ArgumentTypeMismatch(ArgumentTypeMismatch),
	ContextDisallowed(ContextDisallowed),
	UnsupportedStatement(UnsupportedStatement),
	ValueAlreadyExists(ValueAlreadyExists),
	ValueDoesntExist(ValueDoesntExist),
}