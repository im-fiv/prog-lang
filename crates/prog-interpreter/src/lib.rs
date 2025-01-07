mod context;
mod shared;
pub mod value;

pub use context::{Context, ContextFlags};
pub use shared::Shared;
pub use value::{Value, ValueKind};

use anyhow::Result;
use prog_parser::ast;

#[derive(Debug)]
pub struct Interpreter<'ast> {
	ast: ast::Program<'ast>,
	context: Shared<Context>
}

impl<'ast> Interpreter<'ast> {
	pub fn new(ast: ast::Program<'ast>) -> Self {
		Self {
			ast,
			context: Shared::new(Context::new())
		}
	}

	pub fn context(&self) -> std::cell::Ref<'_, Context> { self.context.borrow() }

	pub fn context_mut(&self) -> std::cell::RefMut<'_, Context> { self.context.borrow_mut() }

	// TODO
	pub fn interpret(&mut self) -> Result<Value> { todo!() }
}
