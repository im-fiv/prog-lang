use std::fmt::{self, Display};

use prog_lexer::TokenKind;

use crate::{Primitive, Value};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct List<'ast>(Vec<Value<'ast>>);

impl<'ast> List<'ast> {
	pub fn new(values: Vec<Value<'ast>>) -> Self { Self(values) }
}

impl Primitive for List<'_> {
	fn is_truthy(&self) -> bool { !self.0.is_empty() }
}

// TODO: support for `f.alternate()`
impl Display for List<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let lb = TokenKind::LeftBracket;
		let rb = TokenKind::RightBracket;
		let items = self
			.0
			.iter()
			.map(|i| format!("{i}"))
			.collect::<Vec<_>>()
			.join(", ");

		write!(f, "{lb}{items}{rb}")
	}
}
