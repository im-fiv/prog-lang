mod argument_count_mismatch;
mod argument_type_mismatch;
mod cannot_index_value;
mod context_disallowed;
mod duplicate_object_entry;
mod expression_not_assignable;
mod expression_not_callable;
mod unsupported_binary;
mod unsupported_statement;
mod unsupported_unary;
mod value_already_exists;
mod value_doesnt_exist;

pub use argument_count_mismatch::*;
pub use argument_type_mismatch::*;
pub use cannot_index_value::*;
pub use context_disallowed::*;
pub use duplicate_object_entry::*;
pub use expression_not_assignable::*;
pub use expression_not_callable::*;
pub use unsupported_binary::*;
pub use unsupported_statement::*;
pub use unsupported_unary::*;
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
	CannotIndexValue(CannotIndexValue),
	ContextDisallowed(ContextDisallowed),
	DuplicateObjectEntry(DuplicateObjectEntry),
	ExpressionNotAssignable(ExpressionNotAssignable),
	ExpressionNotCallable(ExpressionNotCallable),
	UnsupportedBinary(UnsupportedBinary),
	UnsupportedStatement(UnsupportedStatement),
	UnsupportedUnary(UnsupportedUnary),
	ValueAlreadyExists(ValueAlreadyExists),
	ValueDoesntExist(ValueDoesntExist),
}