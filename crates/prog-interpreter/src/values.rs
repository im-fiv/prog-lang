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
	Boolean(bool),
	String(String),
	Number(f64),
	List(Vec<RuntimeValue>),
	Object(HashMap<String, RuntimeValue>),

	#[serde(serialize_with = "serde_use_display")]
	Function(RuntimeFunction),
	
	#[serde(serialize_with = "serde_use_display")]
	IntrinsicFunction(IntrinsicFunction),
	
	// It is of type `Identifier` mainly to avoid `TryInto<String>` conflicts with `String` variant in `Conversion` derive macro
	#[serde(skip)]
	Identifier(Identifier),

	Empty
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
		let fmt_list = |f: &mut std::fmt::Formatter<'_>, value: &Vec<RuntimeValue>| {
			let formatted = value
				.iter()
				.map(|entry| entry.to_string())
				.collect::<Vec<String>>()
				.join(", ");
			
			write!(f, "[{formatted}]")
		};

		let fmt_object = |f: &mut std::fmt::Formatter<'_>, value: &HashMap<String, RuntimeValue>| {
			let formatted = value
				.iter()
				.map(|(name, value)| format!("{name} = {value}"))
				.collect::<Vec<String>>()
				.join(", ");
			
			write!(f, "{{ {formatted} }}")
		};
		
		match self {
			Self::Boolean(value) => write!(f, "{}", if value.to_owned() { "true" } else { "false" }),
			Self::String(value) => write!(f, "{value}"),
			Self::Number(value) => write!(f, "{value}"),
			Self::List(value) => fmt_list(f, value),
			Self::Object(value) => fmt_object(f, value),

			Self::Function(value) => write!(f, "{value}"),
			Self::IntrinsicFunction(value) => write!(f, "{value}"),

			Self::Identifier(value) => write!(f, "{}", value.0),
			Self::Empty => write!(f, "")
		}
	}
}

fn serde_use_display<T: Display, S: serde::Serializer>(value: &T, serializer: S) -> std::result::Result<S::Ok, S::Error> {
	serializer.collect_str(value)
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeFunction {
	pub arguments: Vec<String>,
	pub statements: Vec<ast::Statement>
}

impl Display for RuntimeFunction {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let arguments_str = self.arguments.join(", ");
		let formatted = format!("func({arguments_str})");
		
		write!(f, "{{ {formatted} }}")
	}
}

pub type IntrinsicFunctionPtr = fn(&mut RuntimeContext, HashMap<String, ParsedArg>) -> Result<RuntimeValue>;

#[derive(Debug, Clone, PartialEq)]
pub struct IntrinsicFunction {
	pub pointer: IntrinsicFunctionPtr,
	pub arguments: ArgList
}

impl Display for IntrinsicFunction {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "func({:?})", self.pointer)
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Identifier(pub String);

impl From<String> for Identifier {
	fn from(value: String) -> Self {
		Self(value)
	}
}