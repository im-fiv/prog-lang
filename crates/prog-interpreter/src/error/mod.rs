mod arg_count_mismatch;
mod arg_type_mismatch;
mod assertion_eq_failed;
mod assertion_failed;
mod cannot_index_expr;
mod class_field_redef;
mod class_fn_reassign;
mod ctx_disallowed;
mod expr_not_assignable;
mod expr_not_callable;
mod field_doesnt_exist;
mod invalid_class_construction;
mod invalid_extern;
mod invalid_index;
mod obj_entry_redef;
mod unimplemented;
mod var_doesnt_exist;

pub use arg_count_mismatch::ArgCountMismatch;
pub use arg_type_mismatch::ArgTypeMismatch;
pub use assertion_eq_failed::AssertionEqFailed;
pub use assertion_failed::AssertionFailed;
pub use cannot_index_expr::CannotIndexExpr;
pub use class_field_redef::ClassFieldRedef;
pub use class_fn_reassign::ClassFnReassign;
pub use ctx_disallowed::CtxDisallowed;
pub use expr_not_assignable::ExprNotAssignable;
pub use expr_not_callable::ExprNotCallable;
pub use field_doesnt_exist::FieldDoesntExist;
pub use invalid_class_construction::InvalidClassConstruction;
pub use invalid_extern::InvalidExtern;
pub use invalid_index::InvalidIndex;
pub use obj_entry_redef::ObjEntryRedef;
pub use unimplemented::Unimplemented;
pub use var_doesnt_exist::VarDoesntExist;

use prog_utils::pretty_errors::{PrettyError, PrettyErrorKind};

pub type InterpretError<'s> = PrettyError<'s, InterpretErrorKind<'s>>;

#[derive(Debug, Clone, prog_macros::AriadneCompatible)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum InterpretErrorKind<'s> {
	ArgCountMismatch(ArgCountMismatch),
	ArgTypeMismatch(ArgTypeMismatch),
	AssertionEqFailed(AssertionEqFailed<'s>),
	AssertionFailed(AssertionFailed),
	CannotIndexExpr(CannotIndexExpr),
	ClassFieldRedef(ClassFieldRedef<'s>),
	ClassFnReassign(ClassFnReassign),
	CtxDisallowed(CtxDisallowed),
	ExprNotAssignable(ExprNotAssignable),
	ExprNotCallable(ExprNotCallable),
	FieldDoesntExist(FieldDoesntExist),
	InvalidClassConstruction(InvalidClassConstruction),
	InvalidExtern(InvalidExtern),
	InvalidIndex(InvalidIndex<'s>),
	ObjEntryRedef(ObjEntryRedef<'s>),
	Unimplemented(Unimplemented),
	VarDoesntExist(VarDoesntExist)
}

impl<'s> PrettyErrorKind<'s> for InterpretErrorKind<'s> {}
