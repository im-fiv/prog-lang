mod arg_count_mismatch;
mod arg_type_mismatch;
mod assertion_failed;
mod duplicate_obj_entry;
mod expr_not_callable;
mod invalid_extern;
mod unimplemented;
mod var_doesnt_exist;

pub use arg_count_mismatch::ArgCountMismatch;
pub use arg_type_mismatch::ArgTypeMismatch;
pub use assertion_failed::AssertionFailed;
pub use duplicate_obj_entry::DuplicateObjEntry;
pub use expr_not_callable::ExprNotCallable;
pub use invalid_extern::InvalidExtern;
pub use unimplemented::Unimplemented;
pub use var_doesnt_exist::VarDoesntExist;

use prog_utils::pretty_errors::{PrettyError, PrettyErrorKind};

pub type InterpretError<'s> = PrettyError<'s, InterpretErrorKind<'s>>;

#[derive(Debug, Clone, prog_macros::AriadneCompatible)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum InterpretErrorKind<'s> {
	ArgCountMismatch(ArgCountMismatch),
	ArgTypeMismatch(ArgTypeMismatch),
	AssertionFailed(AssertionFailed),
	DuplicateObjEntry(DuplicateObjEntry<'s>),
	ExprNotCallable(ExprNotCallable),
	InvalidExtern(InvalidExtern),
	Unimplemented(Unimplemented),
	VarDoesntExist(VarDoesntExist)
}

impl<'s> PrettyErrorKind<'s> for InterpretErrorKind<'s> {}
