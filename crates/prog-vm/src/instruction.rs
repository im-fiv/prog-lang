use std::fmt::{self, Display};

use prog_macros::extract_fields;
use serde::{Deserialize, Serialize};

use crate::Value;

extract_fields! {
	#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
	pub enum Instruction {
		PUSH(Value),
		POP,
		DUP(usize),
		LOAD(String),
		STORE(String),
		RET,
		NEWFUNC(usize, Vec<Instruction>),
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
		use indent::indent_all_by;

		match self {
			Self::PUSH(inst) => write!(f, "PUSH {}", inst.0),
			Self::POP(_) => write!(f, "POP"),
			Self::DUP(inst) => write!(f, "DUP {}", inst.0),
			Self::LOAD(inst) => write!(f, "LOAD {}", inst.0),
			Self::STORE(inst) => write!(f, "STORE {}", inst.0),
			Self::RET(_) => write!(f, "RET"),
			Self::NEWFUNC(inst) => {
				let strings = inst
					.1
					.iter()
					.map(|inst| format!("{inst}"))
					.collect::<Vec<_>>()
					.join("\n");

				write!(f, "NEWFUNC {}\n{}", inst.0, indent_all_by(4, strings))
			}
			Self::LABEL(inst) => {
				write!(f, "LABEL {} {} {}", inst.name, inst.start, inst.length)
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
