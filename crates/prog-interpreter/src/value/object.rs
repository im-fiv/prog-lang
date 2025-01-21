use std::collections::HashMap;
use std::fmt::{self, Display};

use prog_lexer::TokenKind;

use crate::{Primitive, Shared, Value};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Obj<'ast>(Shared<HashMap<String, Value<'ast>>>);

impl Primitive for Obj<'_> {
	fn is_truthy(&self) -> bool { self.0.borrow().is_empty() }
}

impl<'ast> From<HashMap<String, Value<'ast>>> for Obj<'ast> {
	fn from(entries: HashMap<String, Value<'ast>>) -> Self {
		let container = Shared::new(entries);
		Self::from(container)
	}
}

impl<'ast> From<Shared<HashMap<String, Value<'ast>>>> for Obj<'ast> {
	fn from(container: Shared<HashMap<String, Value<'ast>>>) -> Self { Self(container) }
}

// TODO: support for `f.alternate()`
impl Display for Obj<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let lb = TokenKind::LeftBrace;
		let rb = TokenKind::RightBrace;
		let items = self
			.0
			.borrow()
			.iter()
			.map(|(name, value)| format!("{name} {} {value}", TokenKind::Eq))
			.collect::<Vec<_>>()
			.join(", ");

		write!(f, "{lb} {items} {rb}")
	}
}
