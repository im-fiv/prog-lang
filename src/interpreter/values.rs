use std::fmt::Display;
use anyhow::Result;
use serde::Serialize;

use super::RuntimeContext;
use crate::parser::ast;

pub type IntrinsicFunctionPtr = fn(&mut RuntimeContext, Vec<RuntimeValue>) -> Result<RuntimeValue>;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum RuntimeValue {
	Boolean(bool),
	String(String),
	Number(f64),

	#[serde(skip)]
	Function(RuntimeFunction),
	
	#[serde(skip)]
	IntrinsicFunction(IntrinsicFunctionPtr, i32),
	
	Empty
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeFunction {
	pub arguments: Vec<String>,
	pub statements: Vec<ast::Statement>
}

impl RuntimeValue {
	pub fn kind(&self) -> String {
		match self {
			Self::Boolean(_) => "Boolean",
			Self::String(_) => "String",
			Self::Number(_) => "Number",
			Self::Function(_) => "Function",
			Self::IntrinsicFunction(..) => "IntrinsicFunction",
			Self::Empty => "Void"
		}.to_owned()
	}
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
			Self::String(value) => write!(f, "{value}"),
			Self::Number(value) => write!(f, "{value}"),
			Self::Function(_) => write!(f, "Function"),
			Self::IntrinsicFunction(_, num_args) => write!(f, "IntrinsicFunction({num_args})"),
			Self::Empty => write!(f, "")
		}
	}
}