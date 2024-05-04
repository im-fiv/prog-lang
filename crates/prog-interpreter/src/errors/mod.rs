mod unsupported_statement;
mod value_already_exists;
mod value_doesnt_exist;

pub use unsupported_statement::*;
pub use value_already_exists::*;
pub use value_doesnt_exist::*;

use serde::Serialize;

use prog_utils::pretty_errors::PrettyError;
use prog_macros::ImplAriadneCompatible;

pub type InterpretError = PrettyError<InterpretErrorKind>;

#[derive(Debug, Clone, Serialize, ImplAriadneCompatible)]
pub enum InterpretErrorKind {
	UnsupportedStatement(UnsupportedStatement),
	ValueAlreadyExists(ValueAlreadyExists),
	ValueDoesntExist(ValueDoesntExist)
}