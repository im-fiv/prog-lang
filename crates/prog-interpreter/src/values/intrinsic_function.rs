use std::collections::HashMap;
use std::fmt::{self, Debug, Display};

use anyhow::Result;
use prog_parser::{ast, Position};

use super::Value;
use crate::arg_parser::{ArgList, ParsedArg};
use crate::Interpreter;

#[derive(Clone)]
pub struct RIntrinsicFunction {
	pub pointer: RIntrinsicFunctionPtr,
	pub this: Option<Box<Value>>,
	pub arguments: ArgList
}

#[derive(Debug)]
pub struct RIntrinsicFunctionData<'i, 'p> {
	pub this: Option<Value>,
	pub interpreter: &'i mut Interpreter,
	pub arguments: HashMap<String, ParsedArg>,
	pub call_site: CallSite<'p>
}

pub type RIntrinsicFunctionPtr = fn(RIntrinsicFunctionData) -> Result<Value>;

#[derive(Debug, Clone, PartialEq)]
pub struct CallSite<'p> {
	pub source: String,
	pub file: String,

	pub func: Position,
	pub _lp: Position,
	pub args: Option<ast::Punctuated<'p, Position, Position>>,
	pub _rp: Position
}

impl CallSite<'_> {
	pub fn whole(&self) -> Position {
		let start = self.func.start();
		let end = self._rp.end();

		Position::new(start, end)
	}
}

impl RIntrinsicFunction {
	pub fn new(pointer: RIntrinsicFunctionPtr, arguments: ArgList) -> Self {
		Self {
			pointer,
			this: None,
			arguments
		}
	}

	pub fn call(
		self,
		interpreter: &mut Interpreter,
		arguments: HashMap<String, ParsedArg>,
		call_site: CallSite
	) -> Result<Value> {
		let data = RIntrinsicFunctionData {
			this: self.this.as_deref().cloned(),
			interpreter,
			arguments,
			call_site
		};

		(self.pointer)(data)
	}
}

impl PartialEq for RIntrinsicFunction {
	fn eq(&self, other: &Self) -> bool { self.pointer == other.pointer }
}

impl Debug for RIntrinsicFunction {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { Display::fmt(self, f) }
}

impl Display for RIntrinsicFunction {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let args = format!("{:#?}", self.arguments);
		write!(f, "<intrinsic func({}) @ {:?}>", args, self.pointer)
	}
}
