use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

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

#[derive(Clone, PartialEq)]
pub struct Context {
	inner: Rc<RefCell<InnerContext>>
}

impl Context {
	pub fn new() -> Self {
		Self {
			inner: Rc::new(RefCell::new(InnerContext::new()))
		}
	}

	// Helper functions
	pub fn wrap(inner: InnerContext) -> Self {
		Self {
			inner: Rc::new(RefCell::new(inner))
		}
	}

	pub fn unwrap_or_clone(self) -> InnerContext {
		let cell = Rc::unwrap_or_clone(self.inner);
		cell.take()
	}

	pub fn deref(&self) -> impl Deref<Target = InnerContext> + '_ { self.inner.borrow() }

	pub fn deref_mut(&mut self) -> impl DerefMut<Target = InnerContext> + '_ {
		self.inner.borrow_mut()
	}

	// InnerContext's functions
	pub fn deeper(&mut self) { self.deref_mut().deeper(); }

	pub fn shallower(&mut self) { self.deref_mut().shallower(); }

	pub fn is_subcontext(&self) -> bool { self.deref().is_subcontext() }

	pub fn exists(&self, name: &str) -> bool { self.deref().exists(name) }

	pub fn get(&self, name: &str) -> Result<Value> { self.deref().get(name) }

	pub fn insert(&mut self, name: String, value: Value) -> Option<Value> {
		self.deref_mut().insert(name, value)
	}

	pub fn update(&mut self, name: String, value: Value) -> Result<Value> {
		self.deref_mut().update(name, value)
	}
}

impl Debug for Context {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { Debug::fmt(&self.inner, f) }
}

impl Default for Context {
	fn default() -> Self { Self::new() }
}

#[derive(Clone, PartialEq)]
pub struct InnerContext {
	pub stdin: String,
	pub stdout: String,

	pub flags: ContextFlags,

	pub variables: HashMap<String, Value>,
	pub parent: Option<Context>
}

impl InnerContext {
	pub fn new() -> Self {
		Self {
			stdin: String::new(),
			stdout: String::new(),

			flags: ContextFlags::default(),

			variables: HashMap::new(),
			parent: None
		}
	}

	pub fn deeper(&mut self) {
		let child_context = Self::new();
		let original_context = std::mem::replace(self, child_context);

		// `self` here is already the child context
		self.flags = original_context.flags; // Infer the flags of the original context
		self.parent = Some(Context::wrap(original_context));
	}

	pub fn shallower(&mut self) {
		match self.parent.take() {
			Some(parent) => *self = parent.unwrap_or_clone(),
			None => eprintln!("INTERPRETER WARNING: `InnerContext::shallower()` was called while not having a parent")
		}
	}

	pub fn is_subcontext(&self) -> bool { self.parent.is_some() }

	pub fn exists(&self, name: &str) -> bool {
		if self.variables.contains_key(name) {
			return true;
		}

		match self.parent {
			Some(ref p) => p.deref().exists(name),
			None => false
		}
	}

	pub fn get(&self, name: &str) -> Result<Value> {
		if let Some(var) = self.variables.get(name) {
			return Ok(var.to_owned());
		}

		match self.parent {
			Some(ref p) => p.deref().get(name),
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
					Some(ref mut p) => p.deref_mut().update(name, value),
					None => {
						unreachable!(
							"Match arm reached despite expecting `InnerContext::exists(\"{name}\")` to return `false`"
						)
					}
				}
			}
		}
	}
}

impl Debug for InnerContext {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut debug_struct = f.debug_struct("InnerContext");

		debug_struct.field("flags", &self.flags);
		debug_struct.field("variables", &self.variables);

		if let Some(ref p) = self.parent {
			debug_struct.field("parent", p);
		}

		debug_struct.finish()
	}
}

impl Default for InnerContext {
	fn default() -> Self { Self::new() }
}
