use std::fmt::{self, Display};

use prog_lexer::TokenKind;

use crate::Value;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum CtrlFlow<'ast> {
	Return(Box<Value<'ast>>),
	Break,
	Continue
}

impl Display for CtrlFlow<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Return(v) => write!(f, "{} {}", TokenKind::Return, *v),
			Self::Break => write!(f, "{}", TokenKind::Break),
			Self::Continue => write!(f, "{}", TokenKind::Continue)
		}
	}
}
