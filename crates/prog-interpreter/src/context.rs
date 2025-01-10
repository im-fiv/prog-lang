use std::cell::{Ref, RefMut};
use std::collections::HashMap;

use crate::{Shared, Value};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContextFlags {
	/// Is writing to console stdout allowed?
	/// If `false`, any calls to `print` will only write to [`Interpreter::stdout`].
	pub con_stdout_allowed: bool,
	/// Are `import` calls allowed?
	pub imports_allowed: bool,
	/// Are `input` calls allowed?
	pub inputs_allowed: bool,
	/// Are `extern` expressions allowed?
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

#[derive(Debug, Clone)]
pub struct Context<'ast> {
	inner: Shared<ContextInner<'ast>>
}

impl<'ast> Context<'ast> {
	pub fn new() -> Self {
		Self {
			inner: Shared::new(ContextInner::new())
		}
	}

	pub fn inner<'a>(&'a self) -> Ref<'a, ContextInner<'ast>> { self.inner.borrow() }

	pub fn inner_mut<'a>(&'a self) -> RefMut<'a, ContextInner<'ast>> { self.inner.borrow_mut() }

	pub fn unwrap_or_clone(self) -> ContextInner<'ast>
	where
		ContextInner<'ast>: Clone
	{
		self.inner.unwrap_or_clone()
	}

	pub fn child(&self) -> Self {
		let mut ctx = ContextInner::new();
		ctx.flags = self.inner().flags;
		ctx.parent = Some(self.clone()); // Only the "reference" is cloned

		Self {
			inner: Shared::new(ctx)
		}
	}

	pub fn swap(&mut self, other: Self) -> Self { std::mem::replace(self, other) }

	pub fn exists(&self, name: &str) -> bool {
		if self.inner().vars.contains_key(name) {
			return true;
		}

		match self.inner().parent {
			Some(ref p) => p.exists(name),
			None => false
		}
	}

	pub fn get(&self, name: &str) -> Option<Value<'ast>>
	where
		Value<'ast>: Clone
	{
		if let Some(value) = self.inner().vars.get(name).cloned() {
			return Some(value);
		}

		match self.inner().parent {
			Some(ref p) => p.get(name),
			None => None
		}
	}

	pub fn insert(&self, name: String, value: Value<'ast>) -> Option<Value<'ast>> {
		self.inner_mut().vars.insert(name, value)
	}

	pub fn update(&self, name: String, value: Value<'ast>) -> Option<Value<'ast>> {
		use std::collections::hash_map::Entry;

		let mut inner = self.inner_mut();

		match inner.vars.entry(name.clone()) {
			Entry::Occupied(mut e) => Some(e.insert(value)),
			Entry::Vacant(_) => {
				match inner.parent {
					Some(ref p) => p.update(name, value),
					None => None
				}
			}
		}
	}
}

impl Default for Context<'_> {
	fn default() -> Self { Self::new() }
}

#[derive(Debug, Clone)]
pub struct ContextInner<'ast> {
	pub flags: ContextFlags,

	vars: HashMap<String, Value<'ast>>,
	parent: Option<Context<'ast>>
}

impl<'ast> ContextInner<'ast> {
	pub fn new() -> Self {
		Self {
			flags: ContextFlags::default(),

			vars: HashMap::new(),
			parent: None
		}
	}
}

impl Default for ContextInner<'_> {
	fn default() -> Self { Self::new() }
}
