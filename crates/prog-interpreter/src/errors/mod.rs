mod argument_count_mismatch;
mod argument_type_mismatch;
mod assertion_failed;
mod cannot_index_value;
mod context_disallowed;
mod duplicate_object_entry;
mod expression_not_assignable;
mod expression_not_callable;
mod field_doesnt_exist;
mod function_panicked;
mod unsupported_binary;
mod unsupported_statement;
mod unsupported_unary;
mod variable_doesnt_exist;

pub use argument_count_mismatch::*;
pub use argument_type_mismatch::*;
pub use assertion_failed::*;
pub use cannot_index_value::*;
pub use context_disallowed::*;
pub use duplicate_object_entry::*;
pub use expression_not_assignable::*;
pub use expression_not_callable::*;
pub use field_doesnt_exist::*;
pub use function_panicked::*;
use prog_macros::ImplAriadneCompatible;
use prog_utils::pretty_errors::{PrettyError, PrettyErrorKind};
pub use unsupported_binary::*;
pub use unsupported_statement::*;
pub use unsupported_unary::*;
pub use variable_doesnt_exist::*;

pub type InterpretError = PrettyError<InterpretErrorKind>;

#[derive(Debug, Clone, ImplAriadneCompatible)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum InterpretErrorKind {
	ArgumentCountMismatch(ArgumentCountMismatch),
	ArgumentTypeMismatch(ArgumentTypeMismatch),
	AssertionFailed(AssertionFailed),
	CannotIndexValue(CannotIndexValue),
	ContextDisallowed(ContextDisallowed),
	DuplicateObjectEntry(DuplicateObjectEntry),
	ExpressionNotAssignable(ExpressionNotAssignable),
	ExpressionNotCallable(ExpressionNotCallable),
	FieldDoesntExist(FieldDoesntExist),
	FunctionPanicked(FunctionPanicked),
	UnsupportedBinary(UnsupportedBinary),
	UnsupportedStatement(UnsupportedStatement),
	UnsupportedUnary(UnsupportedUnary),
	VariableDoesntExist(VariableDoesntExist)
}

impl PrettyErrorKind for InterpretErrorKind {}
