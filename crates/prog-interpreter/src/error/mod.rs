mod expression_not_callable;
mod unimplemented;
mod variable_doesnt_exist;

pub use expression_not_callable::ExpressionNotCallable;
pub use unimplemented::Unimplemented;
pub use variable_doesnt_exist::VariableDoesntExist;

use prog_utils::pretty_errors::{PrettyError, PrettyErrorKind};

pub type InterpretError = PrettyError<InterpretErrorKind>;

#[derive(Debug, Clone, prog_macros::AriadneCompatible)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum InterpretErrorKind {
	ExpressionNotCallable(ExpressionNotCallable),
	Unimplemented(Unimplemented),
	VariableDoesntExist(VariableDoesntExist)
}

impl PrettyErrorKind for InterpretErrorKind {}
