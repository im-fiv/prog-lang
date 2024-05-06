mod impls;

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

fn serde_use_display<T: Display, S: serde::Serializer>(value: &T, serializer: S) -> std::result::Result<S::Ok, S::Error> {
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