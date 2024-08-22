use std::collections::HashMap;
use std::io::Write;
use std::vec::IntoIter;

use anyhow::{anyhow, bail, Result};

use crate::instruction::*;
use crate::Value;

pub trait Executable<I> {
	type Output;

	fn execute(&mut self, inst: I) -> Result<Self::Output>;
}

#[derive(Debug)]
pub struct VM {
	pub stack: Vec<Value>,
	pub bindings: HashMap<String, Value>,

	instructions: IntoIter<Instruction>,
	labels: HashMap<String, LABEL>
}

impl VM {
	pub fn new(instructions: Vec<Instruction>) -> Self {
		let instructions = instructions.into_iter();

		Self {
			stack: vec![],
			bindings: HashMap::new(),

			instructions,
			labels: HashMap::new()
		}
	}

	pub fn define_intrinsics(&mut self) {
		{
			fn print_function(vm: &mut VM) -> Result<()> {
				fn error(value: impl std::fmt::Debug) -> anyhow::Error {
					anyhow!(
						"Expected last argument to be a positive whole number, found `{value:?}`"
					)
				}

				let mut arg_count = match vm.execute(POP)? {
					Value::Number(v) => {
						if v.is_sign_negative() || v.fract() != 0.0 {
							return Err(error(v));
						}

						v as usize
					}

					v => return Err(error(v))
				};

				let mut args = vec![];
				while arg_count > 0 {
					args.push(vm.execute(POP)?);
					arg_count -= 1;
				}
				args.reverse();

				let args_str = args
					.into_iter()
					.map(|a| format!("{a}"))
					.collect::<Vec<_>>()
					.join("");
				println!("{args_str}");

				Ok(())
			}

			self.execute(PUSH(Value::IntrinsicFunction {
				arity: None,
				pointer: print_function
			}))
			.unwrap();
			self.execute(STORE("print".to_string())).unwrap();
		}

		{
			fn raw_print_function(vm: &mut VM) -> Result<()> {
				let arg = vm.execute(POP)?;

				print!("{arg}");
				std::io::stdout().flush().unwrap();

				Ok(())
			}

			self.execute(PUSH(Value::IntrinsicFunction {
				arity: Some(1),
				pointer: raw_print_function
			}))
			.unwrap();
			self.execute(STORE("raw_print".to_string())).unwrap()
		}
	}

	pub fn run(&mut self) -> Result<Option<Value>> {
		while let Some(inst) = self.instructions.next() {
			let value = self.execute(inst)?;

			if value.is_some() {
				return Ok(value);
			}
		}

		Ok(None)
	}
}

impl Executable<Instruction> for VM {
	type Output = Option<Value>;

	fn execute(&mut self, inst: Instruction) -> Result<Self::Output> {
		use Instruction as I;

		macro_rules! match_and_discard {
			($value:expr => $($variant:ident)|*) => {
				match $value {
					$(
						I::$variant(inst) => { let _ = self.execute(inst)?; }
					)*

					I::RET(_)
					| I::JMP(_)
					| I::JT(_)
					| I::JTF(_) => unreachable!()
				}
			};
		}

		match inst {
			I::RET(inst) => return Some(self.execute(inst)).transpose(),
			I::JMP(inst) => return self.execute(inst),
			I::JT(inst) => return self.execute(inst),
			I::JTF(inst) => return self.execute(inst),

			_ => {}
		}

		match_and_discard!(inst =>
			DUMPSTACK

			| PUSH
			| POP
			| DUP
			| LOAD
			| STORE
			| NEWFUNC
			| LABEL

			| CALL

			| ADD
			| SUB
			| MUL
			| DIV
			| NEG
			| NOT

			| EQ
			| GT
			| LT
			| GTE
			| LTE
		);

		Ok(None)
	}
}

impl Executable<DUMPSTACK> for VM {
	type Output = ();

	fn execute(&mut self, _inst: DUMPSTACK) -> Result<Self::Output> {
		dbg!(&self.stack);
		Ok(())
	}
}

impl Executable<PUSH> for VM {
	type Output = ();

	fn execute(&mut self, inst: PUSH) -> Result<Self::Output> {
		self.stack.push(inst.0);
		Ok(())
	}
}

impl Executable<POP> for VM {
	type Output = Value;

	fn execute(&mut self, _inst: POP) -> Result<Self::Output> {
		self.stack.pop().ok_or(anyhow!("Stack is empty"))
	}
}

impl Executable<DUP> for VM {
	type Output = ();

	fn execute(&mut self, inst: DUP) -> Result<Self::Output> {
		let value = self.stack.get(inst.0).cloned().ok_or(anyhow!(
			"Value at index {} does not exist (stack length is {})",
			inst.0,
			self.stack.len()
		))?;

		self.execute(PUSH(value))
	}
}

impl Executable<LOAD> for VM {
	type Output = ();

	fn execute(&mut self, inst: LOAD) -> Result<Self::Output> {
		let name = inst.0;
		let value = self
			.bindings
			.get(&name)
			.cloned()
			.ok_or(anyhow!("Binding `{}` does not exist", name))?;

		self.execute(PUSH(value))
	}
}

impl Executable<STORE> for VM {
	type Output = ();

	fn execute(&mut self, inst: STORE) -> Result<Self::Output> {
		let name = inst.0;
		let value = self.execute(POP)?;

		self.bindings.insert(name, value);
		Ok(())
	}
}

impl Executable<RET> for VM {
	type Output = Value;

	fn execute(&mut self, _inst: RET) -> Result<Self::Output> { self.execute(POP) }
}

impl Executable<NEWFUNC> for VM {
	type Output = ();

	fn execute(&mut self, inst: NEWFUNC) -> Result<Self::Output> {
		let arity = inst.0;
		let instructions = inst.1;

		self.execute(PUSH(Value::Function {
			arity,
			instructions
		}))
	}
}

impl Executable<LABEL> for VM {
	type Output = ();

	fn execute(&mut self, inst: LABEL) -> Result<Self::Output> {
		let label_name = inst.0.clone();
		self.labels.insert(label_name, inst);

		Ok(())
	}
}

impl Executable<CALL> for VM {
	type Output = ();

	fn execute(&mut self, _inst: CALL) -> Result<Self::Output> {
		let (arity, instructions) = match self.execute(POP)? {
			Value::Function {
				arity,
				instructions
			} => (arity, instructions),
			Value::IntrinsicFunction { arity, pointer } => {
				if let Some(arity) = arity {
					if self.stack.len() < arity {
						bail!("Expected {} arguments, found {}", arity, self.stack.len());
					}
				}

				return pointer(self);
			}

			v => bail!("Value `{v:?}` is not callable")
		};

		if self.stack.len() < arity {
			bail!("Expected {} arguments, found {}", arity, self.stack.len());
		}

		let mut reversed_args = Vec::with_capacity(arity);
		for _ in 0..arity {
			let arg = self.execute(POP)?;
			reversed_args.push(arg);
		}
		self.stack.extend(reversed_args);

		let prev_instructions = std::mem::replace(&mut self.instructions, instructions.into_iter());

		if let Some(val) = self.run()? {
			self.execute(PUSH(val))?;
		}

		self.instructions = prev_instructions;
		Ok(())
	}
}

impl Executable<JMP> for VM {
	type Output = Option<Value>;

	fn execute(&mut self, inst: JMP) -> Result<Self::Output> {
		let label_name = inst.0;
		let label = self
			.labels
			.get(&label_name)
			.cloned()
			.ok_or(anyhow!("Label with name `{label_name}` does not exist"))?;

		let prev_instructions = std::mem::replace(&mut self.instructions, label.1.into_iter());

		let value = self.run()?;
		self.instructions = prev_instructions;

		Ok(value)
	}
}

impl Executable<JT> for VM {
	type Output = Option<Value>;

	fn execute(&mut self, inst: JT) -> Result<Self::Output> {
		let condition = match self.execute(POP)? {
			Value::Boolean(v) => v,
			v => bail!("Instruction expected a boolean, found `{v:?}`")
		};

		if !condition {
			return Ok(None);
		}

		self.execute(JMP(inst.0))
	}
}

impl Executable<JTF> for VM {
	type Output = Option<Value>;

	fn execute(&mut self, inst: JTF) -> Result<Self::Output> {
		let condition = match self.execute(POP)? {
			Value::Boolean(v) => v,
			v => bail!("Instruction expected a boolean, found `{v:?}`")
		};

		match condition {
			true => self.execute(JMP(inst.0)),
			false => self.execute(JMP(inst.1))
		}
	}
}

impl Executable<ADD> for VM {
	type Output = ();

	fn execute(&mut self, _inst: ADD) -> Result<Self::Output> {
		let rhs = self.execute(POP)?;
		let lhs = self.execute(POP)?;

		let value = (lhs + rhs)?;
		self.execute(PUSH(value))
	}
}

impl Executable<SUB> for VM {
	type Output = ();

	fn execute(&mut self, _inst: SUB) -> Result<Self::Output> {
		let rhs = self.execute(POP)?;
		let lhs = self.execute(POP)?;

		let value = (lhs - rhs)?;
		self.execute(PUSH(value))
	}
}

impl Executable<MUL> for VM {
	type Output = ();

	fn execute(&mut self, _inst: MUL) -> Result<Self::Output> {
		let rhs = self.execute(POP)?;
		let lhs = self.execute(POP)?;

		let value = (lhs * rhs)?;
		self.execute(PUSH(value))
	}
}

impl Executable<DIV> for VM {
	type Output = ();

	fn execute(&mut self, _inst: DIV) -> Result<Self::Output> {
		let rhs = self.execute(POP)?;
		let lhs = self.execute(POP)?;

		let value = (lhs / rhs)?;
		self.execute(PUSH(value))
	}
}

impl Executable<NEG> for VM {
	type Output = ();

	fn execute(&mut self, _inst: NEG) -> Result<Self::Output> {
		let operand = self.execute(POP)?;
		let value = (-operand)?;
		self.execute(PUSH(value))
	}
}

impl Executable<NOT> for VM {
	type Output = ();

	fn execute(&mut self, _inst: NOT) -> Result<Self::Output> {
		let operand = self.execute(POP)?;
		let value = !operand;
		self.execute(PUSH(value))
	}
}

impl Executable<EQ> for VM {
	type Output = ();

	fn execute(&mut self, _inst: EQ) -> Result<Self::Output> {
		let rhs = self.execute(POP)?;
		let lhs = self.execute(POP)?;

		let value = lhs == rhs;
		self.execute(PUSH(Value::Boolean(value)))
	}
}

impl Executable<GT> for VM {
	type Output = ();

	fn execute(&mut self, _inst: GT) -> Result<Self::Output> {
		let rhs = self.execute(POP)?;
		let lhs = self.execute(POP)?;

		let value = lhs.gt(&rhs)?;
		self.execute(PUSH(value))
	}
}

impl Executable<LT> for VM {
	type Output = ();

	fn execute(&mut self, _inst: LT) -> Result<Self::Output> {
		let rhs = self.execute(POP)?;
		let lhs = self.execute(POP)?;

		let value = lhs.lt(&rhs)?;
		self.execute(PUSH(value))
	}
}

impl Executable<GTE> for VM {
	type Output = ();

	fn execute(&mut self, _inst: GTE) -> Result<Self::Output> {
		let rhs = self.execute(POP)?;
		let lhs = self.execute(POP)?;

		let value = lhs.gte(&rhs)?;
		self.execute(PUSH(value))
	}
}

impl Executable<LTE> for VM {
	type Output = ();

	fn execute(&mut self, _inst: LTE) -> Result<Self::Output> {
		let rhs = self.execute(POP)?;
		let lhs = self.execute(POP)?;

		let value = lhs.lte(&rhs)?;
		self.execute(PUSH(value))
	}
}
