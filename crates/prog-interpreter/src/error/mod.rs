mod arg_count_mismatch;
mod arg_type_mismatch;
mod assertion_eq_failed;
mod assertion_failed;
mod cannot_index_expr;
mod ctx_disallowed;
mod duplicate_obj_entry;
mod expr_not_callable;
mod expr_not_assignable;
mod invalid_extern;
mod invalid_index;
mod unimplemented;
mod var_doesnt_exist;

pub use arg_count_mismatch::ArgCountMismatch;
pub use arg_type_mismatch::ArgTypeMismatch;
pub use assertion_eq_failed::AssertionEqFailed;
pub use assertion_failed::AssertionFailed;
pub use cannot_index_expr::CannotIndexExpr;
pub use ctx_disallowed::CtxDisallowed;
pub use duplicate_obj_entry::DuplicateObjEntry;
pub use expr_not_assignable::ExprNotAssignable;
pub use expr_not_callable::ExprNotCallable;
pub use invalid_extern::InvalidExtern;
pub use invalid_index::InvalidIndex;
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
	CtxDisallowed(CtxDisallowed),
	DuplicateObjEntry(DuplicateObjEntry<'s>),
	ExprNotAssignable(ExprNotAssignable),
	ExprNotCallable(ExprNotCallable),
	InvalidExtern(InvalidExtern),
	InvalidIndex(InvalidIndex<'s>),
	Unimplemented(Unimplemented),
	VarDoesntExist(VarDoesntExist)
}

impl<'s> PrettyErrorKind<'s> for InterpretErrorKind<'s> {}
