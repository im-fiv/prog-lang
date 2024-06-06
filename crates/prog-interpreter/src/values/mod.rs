mod boolean;
mod string;
mod number;
mod list;
mod object;
mod function;
mod intrinsic_function;
mod marker;

pub use boolean::*;
pub use string::*;
pub use number::*;
pub use list::*;
pub use object::*;
pub use function::*;
pub use intrinsic_function::*;
pub use marker::*;

use std::fmt::Display;
use std::collections::HashMap;

use prog_parser::ast;
use prog_utils::impl_basic_conv;
use prog_macros::{VariantUnwrap, EnumKind};

#[cfg(feature = "serialize")]
use serde::Serialize;

pub trait RuntimePrimitive {
	type Inner: Clone;

	/// Gets the inner value of the primitive
	fn value(&self) -> &Self::Inner;

	/// Clones the inner value of the primitive
	fn owned(&self) -> Self::Inner {
		self.value().to_owned()
	}

	/// Returns an associated function dispatch map for the type
	fn dispatch_map(&self) -> HashMap<String, IntrinsicFunction>;
}

#[derive(Debug, Clone, PartialEq, VariantUnwrap, EnumKind)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub enum RuntimeValue {
	#[cfg_attr(
		feature = "serialize",
		serde(serialize_with = "s_use_display")
	)]
	Boolean(RuntimeBoolean),

	#[cfg_attr(
		feature = "serialize",
		serde(serialize_with = "s_use_display")
	)]
	String(RuntimeString),

	#[cfg_attr(
		feature = "serialize",
		serde(serialize_with = "s_use_display")
	)]
	Number(RuntimeNumber),

	#[cfg_attr(
		feature = "serialize",
		serde(serialize_with = "s_use_display")
	)]
	List(RuntimeList),

	#[cfg_attr(
		feature = "serialize",
		serde(serialize_with = "s_use_display")
	)]
	Object(RuntimeObject),

	#[cfg_attr(
		feature = "serialize",
		serde(serialize_with = "s_use_display")
	)]
	Function(RuntimeFunction),
	
	#[cfg_attr(
		feature = "serialize",
		serde(serialize_with = "s_use_display")
	)]
	IntrinsicFunction(IntrinsicFunction),

	Empty,

	#[cfg_attr(
		feature = "serialize",
		serde(skip)
	)]
	Identifier(String),

	#[cfg_attr(
		feature = "serialize",
		serde(skip)
	)]
	Marker(MarkerKind)
}

#[cfg(feature = "serialize")]
fn s_use_display<T: Display + Clone, S: serde::Serializer>(value: &T, serializer: S) -> std::result::Result<S::Ok, S::Error> {
	serializer.collect_str(&value)
}

impl_basic_conv!(from RuntimeBoolean => RuntimeValue as Boolean);
impl_basic_conv!(from RuntimeString => RuntimeValue as String);
impl_basic_conv!(from RuntimeNumber => RuntimeValue as Number);
impl_basic_conv!(from RuntimeList => RuntimeValue as List);
impl_basic_conv!(from RuntimeObject => RuntimeValue as Object);
impl_basic_conv!(from RuntimeFunction => RuntimeValue as Function);
impl_basic_conv!(from IntrinsicFunction => RuntimeValue as IntrinsicFunction);
impl_basic_conv!(from MarkerKind => RuntimeValue as Marker);

impl From<ast::expressions::Literal> for RuntimeValue {
	fn from(value: ast::expressions::Literal) -> Self {
		use ast::expressions::Literal;

		match value {
			Literal::Boolean(v, _) => Self::Boolean(v.into()),
			Literal::String(v, _) => Self::String(v.into()),
			Literal::Number(v, _) => Self::Number(v.into())
		}
	}
}

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

			Self::Identifier(value) => write!(f, "{value}"),
			Self::Marker(value) => write!(f, "Marker({value})")
		}
	}
}