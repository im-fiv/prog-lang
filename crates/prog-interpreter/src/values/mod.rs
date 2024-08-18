mod boolean;
mod string;
mod number;
mod list;
mod object;
mod function;
mod intrinsic_function;
mod marker;

use std::collections::HashMap;
use std::fmt::{self, Debug, Display};

pub use boolean::*;
pub use function::*;
pub use intrinsic_function::*;
pub use list::*;
pub use marker::*;
pub use number::*;
pub use object::*;
use prog_macros::{EnumKind, VariantUnwrap};
use prog_parser::ast;
use prog_utils::impl_basic_conv;
#[cfg(feature = "serde")]
use serde::Serialize;
pub use string::*;

pub trait RPrimitive {
	type Inner;

	/// Gets a reference to the inner value of the primitive
	fn get(&self) -> &Self::Inner;

	/// Gets a mutable reference to the inner value of the primitive
	fn get_mut(&mut self) -> &mut Self::Inner;

	/// Clones the inner value of the primitive
	fn get_owned(&self) -> Self::Inner
	where
		Self::Inner: Clone {
		self.get().to_owned()
	}

	/// Returns an associated function dispatch map for the type
	fn dispatch_map(&self) -> HashMap<String, RIntrinsicFunction> { HashMap::new() }
}

//* Note: `Debug` is implemented manually below
#[derive(Clone, PartialEq, VariantUnwrap, EnumKind)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum Value {
	#[cfg_attr(feature = "serde", serde(serialize_with = "s_use_display"))]
	Boolean(RBoolean),

	#[cfg_attr(feature = "serde", serde(serialize_with = "s_use_display"))]
	String(RString),

	#[cfg_attr(feature = "serde", serde(serialize_with = "s_use_display"))]
	Number(RNumber),

	#[cfg_attr(feature = "serde", serde(serialize_with = "s_use_display"))]
	List(RList),

	#[cfg_attr(feature = "serde", serde(serialize_with = "s_use_display"))]
	Object(RObject),

	#[cfg_attr(feature = "serde", serde(serialize_with = "s_use_display"))]
	Function(RFunction),

	#[cfg_attr(feature = "serde", serde(serialize_with = "s_use_display"))]
	IntrinsicFunction(RIntrinsicFunction),

	Empty,

	#[cfg_attr(feature = "serde", serde(skip))]
	Identifier(String),

	#[cfg_attr(feature = "serde", serde(skip))]
	Marker(MarkerKind)
}

#[cfg(feature = "serde")]
fn s_use_display<T: Display + Clone, S: serde::Serializer>(
	value: &T,
	serializer: S
) -> std::result::Result<S::Ok, S::Error> {
	serializer.collect_str(&value)
}

impl Value {
	pub fn is_truthy(&self) -> bool {
		match self {
			// Values that are inexpensive to clone can be cloned
			Self::Boolean(v) => v.get_owned(),
			Self::String(v) => !v.get().is_empty(),
			Self::Number(v) => v.get_owned() != 0.0,
			Self::List(v) => !v.get().is_empty(),
			Self::Object(v) => !v.get().is_empty(),

			Self::Function(_) => true,
			Self::IntrinsicFunction(..) => true,

			Self::Empty => false,

			Self::Identifier(..) => panic!("Got `Value` of kind `Identifier`"),
			Self::Marker(..) => panic!("Got `Value` of kind `Marker`")
		}
	}
}

impl_basic_conv!(from RBoolean => Value as Boolean);
impl_basic_conv!(from RString => Value as String);
impl_basic_conv!(from RNumber => Value as Number);
impl_basic_conv!(from RList => Value as List);
impl_basic_conv!(from RObject => Value as Object);
impl_basic_conv!(from RFunction => Value as Function);
impl_basic_conv!(from RIntrinsicFunction => Value as IntrinsicFunction);
impl_basic_conv!(from MarkerKind => Value as Marker);

impl From<ast::expressions::Literal> for Value {
	fn from(value: ast::expressions::Literal) -> Self {
		use ast::expressions::Literal;

		match value {
			Literal::Boolean(v, _) => Self::Boolean(v.into()),
			Literal::String(v, _) => Self::String(v.into()),
			Literal::Number(v, _) => Self::Number(v.into())
		}
	}
}

impl Debug for Value {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Boolean(bool) => Debug::fmt(bool, f),
			Self::String(str) => Debug::fmt(str, f),
			Self::Number(num) => Debug::fmt(num, f),
			Self::List(list) => Debug::fmt(list, f),
			Self::Object(obj) => Debug::fmt(obj, f),
			Self::Function(func) => Debug::fmt(func, f),
			Self::IntrinsicFunction(func) => Debug::fmt(func, f),

			Self::Empty => write!(f, "none"),

			Self::Identifier(ident) => Debug::fmt(ident, f),
			Self::Marker(marker) => Debug::fmt(marker, f)
		}
	}
}

impl Display for Value {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Boolean(value) => write!(f, "{value}"),
			Self::String(value) => write!(f, "{value}"),
			Self::Number(value) => write!(f, "{value}"),
			Self::List(value) => write!(f, "{value}"),
			Self::Object(value) => write!(f, "{value}"),
			Self::Function(value) => write!(f, "{value}"),
			Self::IntrinsicFunction(value) => write!(f, "{value}"),
			Self::Empty => write!(f, "none"),

			Self::Identifier(value) => write!(f, "{value}"),
			Self::Marker(value) => write!(f, "Marker({value})")
		}
	}
}

impl halloc::Allocatable for Value {}
