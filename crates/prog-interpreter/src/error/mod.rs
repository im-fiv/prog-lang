mod todo;

pub use todo::*;

use prog_utils::pretty_errors::{PrettyError, PrettyErrorKind};

pub type InterpretError = PrettyError<InterpretErrorKind>;

#[derive(Debug, Clone, prog_macros::AriadneCompatible)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum InterpretErrorKind {
	Todo(Todo)
}

impl PrettyErrorKind for InterpretErrorKind {}
