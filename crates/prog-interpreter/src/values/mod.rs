pub mod primitives;

use std::fmt::Display;
use std::collections::HashMap;

use anyhow::Result;
use serde::Serialize;

use prog_parser::ast;
use prog_macros::{VariantUnwrap, EnumKind};

use crate::arg_parser::{ArgList, ParsedArg};
use crate::context::RuntimeContext;

#[derive(Debug, Clone, PartialEq, Serialize, VariantUnwrap, EnumKind)]
pub enum RuntimeValue {
	#[serde(serialize_with = "s_use_display")]
	Boolean(primitives::RuntimeBoolean),

	#[serde(serialize_with = "s_use_display")]
	String(primitives::RuntimeString),

	#[serde(serialize_with = "s_use_display")]
	Number(primitives::RuntimeNumber),

	#[serde(serialize_with = "s_use_display")]
	List(primitives::RuntimeList),

	#[serde(serialize_with = "s_use_display")]
	Object(primitives::RuntimeObject),

	#[serde(serialize_with = "s_use_display")]
	Function(RuntimeFunction),
	
	#[serde(serialize_with = "s_use_display")]
	IntrinsicFunction(IntrinsicFunction),

	Empty,

	// It is of type `Identifier` mainly to avoid `TryInto<String>` conflicts with `String` variant in `Conversion` derive macro
	#[serde(skip)]
	Identifier(Identifier),

	#[serde(skip)]
	Marker(MarkerKind)
}

fn s_use_display<T: Display, S: serde::Serializer>(value: &T, serializer: S) -> std::result::Result<S::Ok, S::Error> {
	serializer.collect_str(value)
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeFunction {
	pub ast: Box<ast::expressions::Function>,
	pub source: String,
	pub file: String
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntrinsicFunction {
	pub pointer: IntrinsicFunctionPtr,
	pub arguments: ArgList
}

pub type IntrinsicFunctionPtr = fn(
	context: &mut RuntimeContext,
	args: HashMap<String, ParsedArg>,
	call_site: CallSite
) -> Result<RuntimeValue>;

#[derive(Debug, Clone, PartialEq)]
pub struct CallSite {
	pub source: String,
	pub file: String,
	pub position: ast::Position
}

#[derive(Debug, Clone, PartialEq)]
pub struct Identifier(pub String);

#[derive(Debug, Clone, PartialEq)]
pub enum MarkerKind {
	Return(Box<RuntimeValue>),
	Break,
	Continue
}

//* From<T> *//

impl From<ast::expressions::Literal> for RuntimeValue {
	fn from(value: ast::expressions::Literal) -> Self {
		use ast::expressions::Literal;

		match value {
			Literal::Boolean(value, _) => Self::Boolean(value.into()),
			Literal::String(value, _) => Self::String(value.into()),
			Literal::Number(value, _) => Self::Number(value.into())
		}
	}
}

impl From<String> for Identifier {
	fn from(value: String) -> Self {
		Self(value)
	}
}

//* Display *//

impl Display for RuntimeValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Boolean(value) => write!(f, "{value}"),
			Self::String(value) => write!(f, "{value}"),
			Self::Number(value) => write!(f, "{value}"),
			Self::List(value) => write!(f, "{value}"),
			Self::Object(value) => write!(f, "{value}"),
			Self::Function(value) => write!(f, "{value}"),
			Self::IntrinsicFunction(value) => write!(f, "{value}"),
			Self::Empty => write!(f, ""),

			Self::Identifier(value) => write!(f, "{}", value.0),
			Self::Marker(value) => write!(f, "Marker({value})")
		}
	}
}

impl Display for RuntimeFunction {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let arguments_str = self
			.ast
			.arguments
			.iter()
			.map(|(a, _)| a.to_owned())
			.collect::<Vec<_>>()
			.join(", ");

		let formatted = format!("Function({arguments_str})");
		write!(f, "{formatted}")
	}
}

impl Display for IntrinsicFunction {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "func({:?})", self.pointer)
	}
}

impl Display for MarkerKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Return(value) => write!(f, "return {value}"),
			Self::Break => write!(f, "break"),
			Self::Continue => write!(f, "continue")
		}
	}
}