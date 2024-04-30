use std::fmt::Display;
use std::collections::HashMap;
use anyhow::Result;
use serde::Serialize;

use prog_parser::ast;
use prog_macros::Conversion;

use crate::arg_parser::{ArgList, ParsedArg};
use crate::context::RuntimeContext;

#[derive(Debug, Clone, PartialEq, Serialize, Conversion)]
pub enum RuntimeValue {
	Boolean(bool),
	String(String),
	Number(f64),

	#[serde(serialize_with = "serialize_function")]
	Function(RuntimeFunction),
	
	#[serde(serialize_with = "serialize_intrinsic_function")]
	IntrinsicFunction(IntrinsicFunction),
	
	Empty
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RuntimeValueKind {
	Boolean,
	String,
	Number,
	Function,
	IntrinsicFunction,
	Empty
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeFunction {
	pub arguments: Vec<String>,
	pub statements: Vec<ast::Statement>
}

pub type IntrinsicFunctionPtr = fn(&mut RuntimeContext, HashMap<String, ParsedArg>) -> Result<RuntimeValue>;

#[derive(Debug, Clone, PartialEq)]
pub struct IntrinsicFunction {
	pub pointer: IntrinsicFunctionPtr,
	pub arguments: ArgList
}

impl RuntimeValue {
	pub fn kind(&self) -> RuntimeValueKind {
		use RuntimeValueKind as Kind;

		match self {
			Self::Boolean(_) => Kind::Boolean,
			Self::String(_) => Kind::String,
			Self::Number(_) => Kind::Number,
			Self::Function(_) => Kind::Function,
			Self::IntrinsicFunction(_) => Kind::IntrinsicFunction,
			Self::Empty => Kind::Empty
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
			Self::IntrinsicFunction(_) => write!(f, "IntrinsicFunction"),
			Self::Empty => write!(f, "")
		}
	}
}

impl Display for RuntimeValueKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Boolean => write!(f, "Boolean"),
			Self::String => write!(f, "String"),
			Self::Number => write!(f, "Number"),
			Self::Function => write!(f, "Function"),
			Self::IntrinsicFunction => write!(f, "IntrinsicFunction"),
			Self::Empty => write!(f, "Nothing")
		}
	}
}

fn serialize_function<S: serde::Serializer>(function: &RuntimeFunction, serializer: S) -> std::result::Result<S::Ok, S::Error> {
	let arguments_str = function.arguments.join(", ");
	let formatted = format!("func({arguments_str})");

	serializer.serialize_str(&formatted[..])
}

fn serialize_intrinsic_function<S: serde::Serializer>(function: &IntrinsicFunction, serializer: S) -> std::result::Result<S::Ok, S::Error> {
	let formatted = format!("func({:p})", function.pointer);
	serializer.serialize_str(&formatted[..])
}