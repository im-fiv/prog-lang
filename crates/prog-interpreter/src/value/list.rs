use std::fmt::{self, Display};

use prog_lexer::TokenKind;

use crate::{Primitive, Shared, Value};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct List<'ast>(Shared<Vec<Value<'ast>>>);

impl<'ast> List<'ast> {
	pub fn get(&self, index: usize) -> Option<Value<'ast>>
	where
		Value<'ast>: Clone
	{
		self.0.borrow().get(index).cloned()
	}

	pub fn insert(&self, index: usize, value: Value<'ast>) -> Option<Value<'ast>> {
		let mut inner_list = self.0.borrow_mut();

		match inner_list.get_mut(index) {
			Some(inner_val) => {
				let previous = std::mem::replace(inner_val, value);
				Some(previous)
			}

			None => {
				if index >= inner_list.len() {
					inner_list.resize(index, Value::None);
				}

				inner_list.insert(index, value);
				None
			}
		}
	}
}

impl Primitive for List<'_> {
	fn is_truthy(&self) -> bool { !self.0.borrow().is_empty() }
}

impl<'ast> From<Vec<Value<'ast>>> for List<'ast> {
	fn from(values: Vec<Value<'ast>>) -> Self { Self(Shared::new(values)) }
}

impl<'ast> From<Shared<Vec<Value<'ast>>>> for List<'ast> {
	fn from(container: Shared<Vec<Value<'ast>>>) -> Self { Self(container) }
}

// TODO: support for `f.alternate()`
impl Display for List<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let lb = TokenKind::LeftBracket;
		let rb = TokenKind::RightBracket;
		let items = self
			.0
			.borrow()
			.iter()
			.map(|i| format!("{i:#}"))
			.collect::<Vec<_>>()
			.join(", ");

		write!(f, "{lb}{items}{rb}")
	}
}
