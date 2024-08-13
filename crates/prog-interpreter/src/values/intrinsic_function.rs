use std::collections::HashMap;
use std::fmt::{self, Debug, Display};

use anyhow::Result;
use prog_parser::ast;

use super::RuntimeValue;
use crate::arg_parser::{ArgList, ParsedArg};
use crate::RuntimeContext;

//* Note: `Debug` is implemented manually below
#[derive(Clone, PartialEq)]
pub struct IntrinsicFunction {
	pub pointer: IntrinsicFunctionPtr,
	pub this: Option<Box<RuntimeValue>>,
	pub arguments: ArgList
}

pub type IntrinsicFunctionPtr = fn(
	this: Option<RuntimeValue>,
	context: &mut RuntimeContext,
	args: HashMap<String, ParsedArg>,
	call_site: CallSite
) -> Result<RuntimeValue>;

#[derive(Debug, Clone, PartialEq)]
pub struct CallSite {
	pub source: String,
	pub file: String,

	pub args_pos: ast::Position,
	pub func_pos: ast::Position,
	pub whole_pos: ast::Position
}

impl IntrinsicFunction {
	pub fn new(pointer: IntrinsicFunctionPtr, arguments: ArgList) -> Self {
		Self {
			pointer,
			this: None,
			arguments
		}
	}

	pub fn call(
		self,
		context: &mut RuntimeContext,
		args: HashMap<String, ParsedArg>,
		call_site: CallSite
	) -> Result<RuntimeValue> {
		(self.pointer)(self.this.map(|this| *this), context, args, call_site)
	}
}

impl Debug for IntrinsicFunction {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { Display::fmt(self, f) }
}

impl Display for IntrinsicFunction {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let args = format!("{:#?}", self.arguments);
		write!(f, "<intrinsic func({}) @ {:?}>", args, self.pointer)
	}
}
