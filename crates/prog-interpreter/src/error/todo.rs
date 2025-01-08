use ariadne::{ColorGenerator, Label};
use prog_utils::pretty_errors::{AriadneCompatible, Span};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Todo;

impl AriadneCompatible for Todo {
	fn message(&self) -> String { String::from("TODO") }

	fn labels(self, span: Span) -> Vec<ariadne::Label<Span>> {
		vec![]
	}
}
