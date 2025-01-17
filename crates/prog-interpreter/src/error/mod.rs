mod duplicate_obj_entry;
mod expr_not_callable;
mod unimplemented;
mod var_doesnt_exist;

pub use duplicate_obj_entry::DuplicateObjEntry;
pub use expr_not_callable::ExprNotCallable;
pub use unimplemented::Unimplemented;
pub use var_doesnt_exist::VarDoesntExist;

use prog_utils::pretty_errors::{PrettyError, PrettyErrorKind};

pub type InterpretError<'s> = PrettyError<'s, InterpretErrorKind<'s>>;

#[derive(Debug, Clone, prog_macros::AriadneCompatible)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum InterpretErrorKind<'s> {
	DuplicateObjEntry(DuplicateObjEntry<'s>),
	ExprNotCallable(ExprNotCallable),
	Unimplemented(Unimplemented),
	VarDoesntExist(VarDoesntExist)
}

impl<'s> PrettyErrorKind<'s> for InterpretErrorKind<'s> {}
