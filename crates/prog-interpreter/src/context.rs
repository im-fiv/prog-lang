use std::collections::HashMap;
use std::fmt::{self, Debug};

use anyhow::{bail, Result};

use crate::Value;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContextFlags {
	pub con_stdout_allowed: bool,
	pub imports_allowed: bool,
	pub inputs_allowed: bool,
	pub externs_allowed: bool
}

impl Default for ContextFlags {
	fn default() -> Self {
		Self {
			con_stdout_allowed: true,
			imports_allowed: true,
			inputs_allowed: true,
			externs_allowed: true
		}
	}
}

//* Note: `Debug` is implemented manually below
#[derive(Clone, PartialEq)]
pub struct Context {
	pub stdin: String,
	pub stdout: String,

	pub flags: ContextFlags,

	pub variables: HashMap<String, Value>,
	pub parent: Option<Box<Self>>
}

impl Context {
	pub fn new() -> Self {
		Self {
			stdin: String::new(),
			stdout: String::new(),

			flags: ContextFlags::default(),

			variables: HashMap::new(),
			parent: None
		}
	}

	pub fn level(&self) -> usize {
		let mut level = 0;
		let mut current = &self.parent;

		while let Some(ref ctx) = current {
			level += 1;
			current = &ctx.parent;
		}

		level
	}

	pub fn deeper(&mut self) {
		let child_context = Self::new();
		let original_context = std::mem::replace(self, child_context);

		// `self` here is already the child context
		self.flags = original_context.flags; // Infer the flags of the original context
		self.parent = Some(Box::new(original_context));
	}

	pub fn shallower(&mut self) {
		match self.parent.take() {
			Some(parent) => *self = *parent,
			None => eprintln!("Warning `Context::shallower()` was called while not having a parent")
		}
	}

	pub fn is_subcontext(&self) -> bool { self.parent.is_some() }

	pub fn exists(&self, name: &str) -> bool {
		if self.variables.contains_key(name) {
			return true;
		}

		match self.parent {
			Some(ref p) => p.exists(name),
			None => false
		}
	}

	pub fn get(&self, name: &str) -> Result<Value> {
		if let Some(var) = self.variables.get(name) {
			return Ok(var.to_owned());
		}

		match self.parent {
			Some(ref p) => p.get(name),
			None => bail!("Variable with name `{name}` does not exist")
		}
	}

	pub fn get_mut(&mut self, name: &str) -> Result<&mut Value> {
		if let Some(var) = self.variables.get_mut(name) {
			return Ok(var);
		}

		match self.parent {
			Some(ref mut p) => p.get_mut(name),
			None => bail!("Variable with name `{name}` does not exist")
		}
	}

	pub fn insert(&mut self, name: String, value: Value) -> Option<Value> {
		self.variables.insert(name, value)
	}

	pub fn update(&mut self, name: String, value: Value) -> Result<Value> {
		use std::collections::hash_map::Entry;

		if !self.exists(&name) {
			bail!("Variable with name `{name}` does not exist");
		}

		match self.variables.entry(name.clone()) {
			Entry::Occupied(mut e) => Ok(e.insert(value)),
			Entry::Vacant(_) => {
				match self.parent {
					Some(ref mut p) => p.update(name, value),
					None => {
						unreachable!(
							"Match arm reached despite expecting `Context::exists(\"{name}\")` to return `false`"
						)
					}
				}
			}
		}
	}
}

impl Debug for Context {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut debug_struct = f.debug_struct("Context");

		debug_struct.field("flags", &self.flags);
		debug_struct.field("variables", &self.variables);

		if let Some(ref p) = self.parent {
			debug_struct.field("parent", p);
		}

		debug_struct.finish()
	}
}

impl Default for Context {
	fn default() -> Self { Self::new() }
}

impl halloc::Allocatable for Context {}
