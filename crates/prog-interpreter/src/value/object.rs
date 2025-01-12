use std::fmt::{self, Display};
use std::collections::HashMap;

use prog_lexer::TokenKind;

use crate::{Primitive, Value};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Obj<'ast>(HashMap<String, Value<'ast>>);

impl<'ast> Obj<'ast> {
	pub fn new(value: HashMap<String, Value<'ast>>) -> Self { Self(value) }
}

impl Primitive for Obj<'_> {
	fn is_truthy(&self) -> bool { !self.0.is_empty() }
}

// TODO: support for `f.alternate()`
impl Display for Obj<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let lb = TokenKind::LeftBrace;
		let rb = TokenKind::RightBrace;
		let items = self
			.0
			.iter()
			.map(|(name, value)| format!("{name} {} {value}", TokenKind::Eq))
			.collect::<Vec<_>>()
			.join(", ");

		write!(f, "{lb} {items} {rb}")
	}
}
