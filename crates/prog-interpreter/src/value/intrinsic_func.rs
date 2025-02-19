use std::borrow::Cow;
use std::fmt::{self, Debug, Display};
use std::rc::Rc;

use crate::arg_parser::ArgList;
use crate::{Callable, CallableData, InterpretResult, Primitive, Value};

pub type IntrinsicFnPtr<'int> =
	for<'intref> fn(CallableData<'intref, 'int>) -> InterpretResult<'int, Value<'int>>;

#[derive(Debug, Clone)]
pub struct IntrinsicFn<'int> {
	pub(crate) ptr: IntrinsicFnPtr<'int>,
	pub(crate) args: Rc<ArgList>
}

impl<'int> IntrinsicFn<'int> {
	pub(crate) fn new(ptr: IntrinsicFnPtr<'int>, args: ArgList) -> Self {
		Self {
			ptr,
			args: Rc::new(args)
		}
	}

	pub fn address(&self) -> usize { self.ptr as usize }
}

impl Primitive for IntrinsicFn<'_> {
	fn is_truthy(&self) -> bool { true }
}

impl<'intref, 'int: 'intref> Callable<'intref, 'int> for IntrinsicFn<'int> {
	fn arg_list(&self) -> Cow<ArgList> { Cow::Borrowed(&self.args) }

	fn call(
		&mut self,
		data: CallableData<'intref, 'int>
	) -> crate::InterpretResult<'int, Value<'int>> {
		(self.ptr)(data)
	}
}

impl PartialEq for IntrinsicFn<'_> {
	fn eq(&self, other: &Self) -> bool { self.address() == other.address() }
}

impl Display for IntrinsicFn<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "<intrinsic func() @ {:?}>", self.ptr)
	}
}

#[cfg(feature = "serde")]
impl serde::Serialize for IntrinsicFn<'_> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer
	{
		let address = format!("{:?}", self.ptr);
		serializer.serialize_newtype_struct("IntrinsicFn", &address)
	}
}
