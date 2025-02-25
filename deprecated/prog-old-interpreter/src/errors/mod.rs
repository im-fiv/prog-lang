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
mod non_exhaustive_class_construction;
mod cannot_reassign_class_functions;
mod invalid_extern_argument;
mod non_existent_extern_item;
mod invalid_file;

pub use argument_count_mismatch::*;
pub use argument_type_mismatch::*;
pub use assertion_failed::*;
pub use cannot_index_value::*;
pub use cannot_reassign_class_functions::*;
pub use context_disallowed::*;
pub use duplicate_object_entry::*;
pub use expression_not_assignable::*;
pub use expression_not_callable::*;
pub use field_doesnt_exist::*;
pub use function_panicked::*;
pub use invalid_extern_argument::*;
pub use invalid_file::*;
pub use non_exhaustive_class_construction::*;
pub use non_existent_extern_item::*;
pub use unsupported_binary::*;
pub use unsupported_statement::*;
pub use unsupported_unary::*;
pub use variable_doesnt_exist::*;

use prog_macros::ImplAriadneCompatible;
use prog_utils::pretty_errors::{PrettyError, PrettyErrorKind};

pub type InterpretError = PrettyError<InterpretErrorKind>;

#[derive(Debug, Clone, ImplAriadneCompatible)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum InterpretErrorKind {
	ArgumentCountMismatch(ArgumentCountMismatch),
	ArgumentTypeMismatch(ArgumentTypeMismatch),
	AssertionFailed(AssertionFailed),
	CannotIndexValue(CannotIndexValue),
	CannotReassignClassFunctions(CannotReassignClassFunctions),
	ContextDisallowed(ContextDisallowed),
	DuplicateObjectEntry(DuplicateObjectEntry),
	ExpressionNotAssignable(ExpressionNotAssignable),
	ExpressionNotCallable(ExpressionNotCallable),
	FieldDoesntExist(FieldDoesntExist),
	FunctionPanicked(FunctionPanicked),
	InvalidExternArgument(InvalidExternArgument),
	InvalidFile(InvalidFile),
	NonExhaustiveClassConstruction(NonExhaustiveClassConstruction),
	NonExistentExternItem(NonExistentExternItem),
	UnsupportedBinary(UnsupportedBinary),
	UnsupportedStatement(UnsupportedStatement),
	UnsupportedUnary(UnsupportedUnary),
	VariableDoesntExist(VariableDoesntExist)
}

impl PrettyErrorKind for InterpretErrorKind {}
