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
use std::cell::RefCell;

use serde::Serialize;

use prog_parser::ast;
use prog_utils::impl_basic_conv;
use prog_macros::{VariantUnwrap, EnumKind};

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

#[derive(Debug, Clone, PartialEq, Serialize, VariantUnwrap, EnumKind)]
pub enum RuntimeValue {
	#[serde(serialize_with = "s_use_display")]
	Boolean(RefCell<RuntimeBoolean>),

	#[serde(serialize_with = "s_use_display")]
	String(RefCell<RuntimeString>),

	#[serde(serialize_with = "s_use_display")]
	Number(RefCell<RuntimeNumber>),

	#[serde(serialize_with = "s_use_display")]
	List(RefCell<RuntimeList>),

	#[serde(serialize_with = "s_use_display")]
	Object(RefCell<RuntimeObject>),

	#[serde(serialize_with = "s_use_display")]
	Function(RefCell<RuntimeFunction>),
	
	#[serde(serialize_with = "s_use_display")]
	IntrinsicFunction(RefCell<IntrinsicFunction>),

	Empty,

	#[serde(skip)]
	Identifier(String),

	#[serde(skip)]
	Marker(MarkerKind)
}

fn s_use_display<T: Display + Clone, S: serde::Serializer>(value: &RefCell<T>, serializer: S) -> std::result::Result<S::Ok, S::Error> {
	let owned = value.borrow().to_owned();
	serializer.collect_str(&owned)
}

impl_basic_conv!(from RefCell<RuntimeBoolean> => RuntimeValue as Boolean);
impl_basic_conv!(from RefCell<RuntimeString> => RuntimeValue as String);
impl_basic_conv!(from RefCell<RuntimeNumber> => RuntimeValue as Number);
impl_basic_conv!(from RefCell<RuntimeList> => RuntimeValue as List);
impl_basic_conv!(from RefCell<RuntimeObject> => RuntimeValue as Object);

impl_basic_conv!(from RuntimeBoolean => RuntimeValue as Boolean { RefCell::new });
impl_basic_conv!(from RuntimeString => RuntimeValue as String { RefCell::new });
impl_basic_conv!(from RuntimeNumber => RuntimeValue as Number { RefCell::new });
impl_basic_conv!(from RuntimeList => RuntimeValue as List { RefCell::new });
impl_basic_conv!(from RuntimeObject => RuntimeValue as Object { RefCell::new });
impl_basic_conv!(from RuntimeFunction => RuntimeValue as Function { RefCell::new });
impl_basic_conv!(from IntrinsicFunction => RuntimeValue as IntrinsicFunction { RefCell::new });
impl_basic_conv!(from MarkerKind => RuntimeValue as Marker);

impl From<ast::expressions::Literal> for RuntimeValue {
	fn from(value: ast::expressions::Literal) -> Self {
		use ast::expressions::Literal;

		match value {
			Literal::Boolean(value, _) => Self::Boolean(RuntimeBoolean::from(value).into()),
			Literal::String(value, _) => Self::String(RuntimeString::from(value).into()),
			Literal::Number(value, _) => Self::Number(RuntimeNumber::from(value).into())
		}
	}
}

impl Display for RuntimeValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		fn borrow_and_write<T: Display>(f: &mut std::fmt::Formatter<'_>, value: &RefCell<T>) -> std::fmt::Result {
			let inner = value.borrow();
			write!(f, "{inner}")
		}
			
		match self {
			Self::Boolean(value) => borrow_and_write(f, value),
			Self::String(value) => borrow_and_write(f, value),
			Self::Number(value) => borrow_and_write(f, value),
			Self::List(value) => borrow_and_write(f, value),
			Self::Object(value) => borrow_and_write(f, value),
			Self::Function(value) => borrow_and_write(f, value),
			Self::IntrinsicFunction(value) => borrow_and_write(f, value),
			Self::Empty => write!(f, ""),

			Self::Identifier(value) => write!(f, "{value}"),
			Self::Marker(value) => write!(f, "Marker({value})")
		}
	}
}