use std::fmt::{self, Display};

use indent::indent_all_by;
use prog_macros::extract_fields;
use serde::{Deserialize, Serialize};

use crate::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bytecode {
	pub(crate) instructions: Vec<Instruction>
}

impl Bytecode {
	pub fn new(instructions: Vec<Instruction>) -> Self { Self { instructions } }

	pub fn as_bytes(&self) -> bincode::Result<Vec<u8>> { bincode::serialize(&self) }

	pub fn from_bytes(bytes: &[u8]) -> bincode::Result<Self> { bincode::deserialize(bytes) }
}

impl Display for Bytecode {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		macro_rules! nested_instruction {
			($index:ident, $length:expr) => {{
				let start = $index + 1;
				let end = start + $length;
				let inner_instructions = &self.instructions[start..end];

				let inner_bytecode = Self {
					instructions: inner_instructions.to_vec()
				};

				let formatted_inner = format!("{inner_bytecode}");
				write!(f, "{}", indent_all_by(4, formatted_inner))?;

				$index = end - 1;
			}};
		}

		let mut index = 0;

		while index < self.instructions.len() {
			let inst = &self.instructions[index];
			write!(f, "{inst}\n")?;

			match inst {
				Instruction::LABEL(inst) => nested_instruction!(index, inst.length),
				Instruction::NEWFUNC(inst) => nested_instruction!(index, inst.length),

				_ => {}
			}

			index = index + 1;
		}

		Ok(())
	}
}

extract_fields! {
	#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
	pub enum Instruction {
		PUSH(Value),
		POP,
		DUP(usize),
		LOAD(String),
		STORE(String),
		RET,
		NEWFUNC {
			arity: usize,
			start: usize,
			length: usize
		},
		LABEL {
			name: String,
			start: usize,
			length: usize
		},

		CALL,
		/// Unconditional jump
		JMP(String),
		/// Jump if true
		JT(String),
		/// Jump to `0` if true, otherwise jump to `1`
		JTF(String, String),

		ADD,
		SUB,
		MUL,
		DIV,
		NEG,
		NOT,

		EQ,
		GT,
		LT,
		GTE,
		LTE
	}
}

impl Display for Instruction {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::PUSH(inst) => write!(f, "PUSH {}", inst.0),
			Self::POP(_) => write!(f, "POP"),
			Self::DUP(inst) => write!(f, "DUP {}", inst.0),
			Self::LOAD(inst) => write!(f, "LOAD {}", inst.0),
			Self::STORE(inst) => write!(f, "STORE {}", inst.0),
			Self::RET(_) => write!(f, "RET"),
			Self::NEWFUNC(inst) => {
				// TODO: inst.start
				write!(f, "NEWFUNC {} _ {}", inst.arity, inst.length)
			}
			Self::LABEL(inst) => {
				// TODO: inst.start
				write!(f, "LABEL {} _ {}", inst.name, inst.length)
			}

			Self::CALL(_) => write!(f, "CALL"),
			Self::JMP(inst) => write!(f, "JMP {}", inst.0),
			Self::JT(inst) => write!(f, "JT {}", inst.0),
			Self::JTF(inst) => write!(f, "JTF {} {}", inst.0, inst.1),

			Self::ADD(_) => write!(f, "ADD"),
			Self::SUB(_) => write!(f, "SUB"),
			Self::MUL(_) => write!(f, "MUL"),
			Self::DIV(_) => write!(f, "DIV"),
			Self::NEG(_) => write!(f, "NEG"),
			Self::NOT(_) => write!(f, "NOT"),

			Self::EQ(_) => write!(f, "EQ"),
			Self::GT(_) => write!(f, "GT"),
			Self::LT(_) => write!(f, "LT"),
			Self::GTE(_) => write!(f, "GTE"),
			Self::LTE(_) => write!(f, "LTE")
		}
	}
}

impl LABEL {
	pub const fn end(&self) -> usize { self.start + self.length }
}
