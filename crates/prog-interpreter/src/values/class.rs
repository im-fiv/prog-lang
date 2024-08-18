use std::collections::HashMap;
use std::fmt::{self, Display};

use halloc::HeapMutator;

use super::Value;

// TODO: implement `Debug` manually
//* Note: `PartialEq` is implemented manually below
#[derive(Debug, Clone)]
pub struct RClass {
	pub name: String,
	pub fields: HeapMutator<'static, HashMap<String, Value>>
}

impl Display for RClass {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "class {} {{ ", self.name)?;

		let mut fields_str = Vec::with_capacity(self.fields.len());
		for (name, value) in self.fields.iter() {
			fields_str.push(format!("{name} = {value}"));
		}
		write!(f, "{}", fields_str.join(", "))?;

		write!(f, " }}")
	}
}

impl PartialEq for RClass {
	fn eq(&self, other: &Self) -> bool {
		let name = self.name == other.name;
		let fields = self.fields.get() == other.fields.get();

		name && fields
	}
}

// Same as with Object
impl Drop for RClass {
	fn drop(&mut self) {
		if !self.fields.can_dealloc() {
			return;
		}

		for (_, value) in self.fields.drain() {
			// `name` will be dropped automatically
			drop(value);
		}
	}
}

// TODO: implement `Debug` manually
//* Note: `PartialEq` is implemented manually below
#[derive(Debug, Clone)]
pub struct RClassInstance {
	pub class: RClass,
	pub fields: HeapMutator<'static, HashMap<String, Value>>
}

impl PartialEq for RClassInstance {
	fn eq(&self, other: &Self) -> bool {
		let class = self.class == other.class;
		let fields = self.fields.get() == other.fields.get();

		class && fields
	}
}

impl Display for RClassInstance {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "instance {} {{ ", self.class.name)?;

		let mut fields_str = Vec::with_capacity(self.fields.len());
		for (name, value) in self.fields.iter() {
			fields_str.push(format!("{name} = {value}"));
		}
		write!(f, "{}", fields_str.join(", "))?;

		write!(f, " }}")
	}
}

// Same as with Object
impl Drop for RClassInstance {
	fn drop(&mut self) {
		if !self.fields.can_dealloc() {
			return;
		}

		for (_, value) in self.fields.drain() {
			// `name` will be dropped automatically
			drop(value);
		}
	}
}
