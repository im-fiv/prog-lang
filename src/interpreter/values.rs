use std::fmt::Display;
use crate::parser::ast;

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
	Boolean(bool),
	String(String),
	Number(f64),
	Function(RuntimeFunction),
	Empty
}

// TODO
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeFunction {
	pub arguments: Vec<String>,
	pub statements: Vec<ast::Statement>
}

impl From<ast::expressions::Literal> for RuntimeValue {
	fn from(value: ast::expressions::Literal) -> Self {
		use ast::expressions::Literal;

		match value {
			Literal::Boolean(value) => Self::Boolean(value),
			Literal::String(value) => Self::String(value),
			Literal::Number(value) => Self::Number(value)
		}
	}
}

impl Display for RuntimeValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Boolean(value) => write!(f, "{}", if value.to_owned() { "true" } else { "false" }),
			Self::String(value) => write!(f, "\"{value}\""),
			Self::Number(value) => write!(f, "{value}"),
			Self::Function(_) => write!(f, "RuntimeFunction"),
			Self::Empty => write!(f, "")
		}
	}
}