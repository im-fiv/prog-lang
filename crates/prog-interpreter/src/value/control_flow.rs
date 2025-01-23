use std::fmt::{self, Display};

use prog_lexer::TokenKind;
use prog_utils::pretty_errors::Span;

use crate::Value;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum CtrlFlow<'ast> {
	Return(Span<'ast>, Box<Value<'ast>>),
	Break(Span<'ast>),
	Continue(Span<'ast>)
}

impl<'ast> CtrlFlow<'ast> {
	pub fn span<'a>(&'a self) -> Span<'ast> {
		match self {
			Self::Return(s, _) => *s,
			Self::Break(s) => *s,
			Self::Continue(s) => *s
		}
	}
}

impl Display for CtrlFlow<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Return(_, v) => write!(f, "{} {}", TokenKind::Return, *v),
			Self::Break(_) => write!(f, "{}", TokenKind::Break),
			Self::Continue(_) => write!(f, "{}", TokenKind::Continue)
		}
	}
}
