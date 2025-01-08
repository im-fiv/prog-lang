mod context;
pub mod error;
mod shared;
pub mod value;

pub use context::{Context, ContextFlags};
pub use error::{InterpretError, InterpretErrorKind};
pub use shared::Shared;
pub use value::{Value, ValueKind};

use prog_parser::ast;

pub type InterpretResult<T> = Result<T, InterpretError>;

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
	pub fn interpret(&mut self) -> InterpretResult<Value> { Ok(Value::Empty) }
}
