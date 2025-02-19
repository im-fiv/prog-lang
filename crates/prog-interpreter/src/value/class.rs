use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fmt::{self, Display};

use prog_lexer::TokenKind;
use prog_macros::get_argument;
use prog_parser::ASTNode;

use crate::arg_parser::{Arg, ArgList};
use crate::{
	error, Callable, CallableData, InterpretError, InterpretErrorKind, InterpretResult, Primitive,
	Shared, Value, ValueKind
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Class<'i> {
	name: String,
	fields: Shared<HashMap<String, Value<'i>>>
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ClassInstance<'i> {
	class: Class<'i>,
	fields: Shared<HashMap<String, Value<'i>>>
}

impl<'i> Class<'i> {
	pub(crate) fn new(name: String, fields: Shared<HashMap<String, Value<'i>>>) -> Self {
		Self { name, fields }
	}

	pub fn name(&self) -> &str { &self.name }

	pub fn contains<N>(&self, name: N) -> bool
	where
		N: AsRef<str>
	{
		self.fields.borrow().contains_key(name.as_ref())
	}

	pub fn get<N>(&self, name: N) -> Option<Value<'i>>
	where
		Value<'i>: Clone,
		N: AsRef<str>
	{
		self.fields.borrow().get(name.as_ref()).cloned()
	}

	pub(crate) fn uninits(&self) -> HashSet<String> {
		self.fields
			.borrow()
			.iter()
			.filter(|&(_, val)| val.kind() == ValueKind::None)
			.map(|(name, _)| name.to_owned())
			.collect()
	}
}

impl<'i> ClassInstance<'i> {
	pub(crate) fn new(class: Class<'i>, fields: Shared<HashMap<String, Value<'i>>>) -> Self {
		Self { class, fields }
	}

	pub fn name(&self) -> &str { self.class.name() }

	pub fn contains<N>(&self, name: N) -> bool
	where
		N: AsRef<str>
	{
		self.fields.borrow().contains_key(name.as_ref())
	}

	pub fn get<N>(&self, name: N) -> Option<Value<'i>>
	where
		Value<'i>: Clone,
		N: AsRef<str>
	{
		self.fields
			.borrow()
			.get(name.as_ref())
			.cloned()
			.or(self.class.get(name))
	}

	pub fn insert<N>(&self, name: N, value: Value<'i>) -> Option<Value<'i>>
	where
		N: Into<String>
	{
		self.fields.borrow_mut().insert(name.into(), value)
	}
}

impl<'intref, 'int: 'intref> Callable<'intref, 'int> for Class<'int> {
	fn arg_list(&self) -> Cow<crate::arg_parser::ArgList> {
		if self.uninits().is_empty() {
			return Cow::Owned(ArgList::new_empty());
		}

		Cow::Owned(ArgList::new(vec![Arg::Required(
			"fields".into(),
			ValueKind::Obj
		)]))
	}

	fn call(
		&mut self,
		CallableData {
			mut args,
			call_site,
			..
		}: CallableData<'intref, 'int>
	) -> InterpretResult<'int, Value<'int>> {
		let args = get_argument!(args => fields: Obj?).unwrap_or_default();

		let mut fields = HashMap::new();
		let mut uninits = self.uninits();

		for (name, value) in args.unwrap_or_clone() {
			if !uninits.remove(&name) {
				// We have no way of finding the exact entry in the object
				return Err(InterpretError::new(
					call_site.args.span(),
					InterpretErrorKind::InvalidClassConstruction(
						error::InvalidClassConstruction::UnknownField {
							class_name: self.name().to_owned(),
							field_name: name
						}
					)
				));
			}

			fields.insert(name, value);
		}

		if !uninits.is_empty() {
			return Err(InterpretError::new(
				call_site.args.span(),
				InterpretErrorKind::InvalidClassConstruction(
					error::InvalidClassConstruction::MissingFields {
						class_name: self.name().to_owned(),
						field_names: uninits.into_iter().collect()
					}
				)
			));
		}

		let instance = ClassInstance::new(self.clone(), Shared::new(fields));
		Ok(Value::ClassInstance(instance))
	}
}

impl Primitive for Class<'_> {
	fn is_truthy(&self) -> bool { true }
}

impl Primitive for ClassInstance<'_> {
	fn is_truthy(&self) -> bool { true }
}

impl Display for Class<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		// TODO: exhaustive formatting
		write!(f, "{} {}", TokenKind::Class, self.name())
	}
}

impl Display for ClassInstance<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		// TODO: exhaustive formatting
		write!(f, "instance {}", self.name())
	}
}
