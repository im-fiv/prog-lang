pub mod instruction;
mod value;
mod vm;

pub use instruction::{Bytecode, Instruction};
pub use value::Value;
pub use vm::VM;
