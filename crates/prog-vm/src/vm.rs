use std::collections::HashMap;
use std::io::Write;
use std::vec::IntoIter;

use anyhow::{anyhow, bail, Result};

use crate::instruction::*;
use crate::Value;

fn create_intrinsics() -> HashMap<String, Value> {
	let mut entries = vec![];

	entries.push({
		fn print_function(vm: &mut VM) -> Result<()> {
			fn error(value: impl std::fmt::Debug) -> anyhow::Error {
				anyhow!(
					"Expected last argument to be a positive whole number, found `{value:?}`"
				)
			}

			let mut arg_count = match vm.execute_pop(POP)? {
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
				args.push(vm.execute_pop(POP)?);
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

		("print".to_string(), Value::IntrinsicFunction {
			arity: None,
			pointer: print_function
		})
	});

	entries.push({
		fn raw_print_function(vm: &mut VM) -> Result<()> {
			let arg = vm.execute_pop(POP)?;

			print!("{arg}");
			std::io::stdout().flush().unwrap();

			Ok(())
		}


		("raw_print".to_string(), Value::IntrinsicFunction {
			arity: Some(1),
			pointer: raw_print_function
		})
	});

	let mut map = HashMap::new();
	map.extend(entries);
	map
}

#[derive(Debug)]
pub struct VM {
	pub stack: Vec<Value>,
	pub bindings: HashMap<String, Value>,

	instructions: IntoIter<Instruction>,
	labels: HashMap<String, LABEL>
}

impl VM {
	pub fn new(instructions: Vec<Instruction>) -> Result<Self> {
		let instructions = instructions.into_iter();

		let mut this = Self {
			stack: Vec::with_capacity(2_usize.pow(16)),
			bindings: HashMap::new(),

			instructions,
			labels: HashMap::new()
		};
		this.define_intrinsics()?;

		Ok(this)
	}

	fn define_intrinsics(&mut self) -> Result<()> {
		for (name, value) in create_intrinsics() {
			self.execute_push(PUSH(value))?;
			self.execute_store(STORE(name))?;
		}

		Ok(())
	}

	pub fn run(&mut self) -> Result<Option<Value>> {
		while let Some(inst) = self.instructions.next() {
			let value = self.execute_instruction(inst)?;

			if value.is_some() {
				return Ok(value);
			}
		}

		Ok(None)
	}

	fn execute_instruction(&mut self, inst: Instruction) -> Result<Option<Value>> {
		use Instruction as I;

		match inst {
			I::RET(inst) => return Some(self.execute_ret(inst)).transpose(),
			I::JMP(inst) => return self.execute_jmp(inst),
			I::JT(inst) => return self.execute_jt(inst),
			I::JTF(inst) => return self.execute_jtf(inst),

			_ => {}
		}

		match inst {
			I::PUSH(inst) => self.execute_push(inst),
			I::POP(inst) => self.execute_pop(inst).map(|_| ()),
			I::DUP(inst) => self.execute_dup(inst),
			I::LOAD(inst) => self.execute_load(inst),
			I::STORE(inst) => self.execute_store(inst),
			I::RET(_) => unreachable!(),
			I::NEWFUNC(inst) => self.execute_newfunc(inst),
			I::LABEL(inst) => self.execute_label(inst),

			I::CALL(inst) => self.execute_call(inst),
			I::JMP(_) => unreachable!(),
			I::JT(_) => unreachable!(),
			I::JTF(_) => unreachable!(),

			I::ADD(inst) => self.execute_add(inst),
			I::SUB(inst) => self.execute_sub(inst),
			I::MUL(inst) => self.execute_mul(inst),
			I::DIV(inst) => self.execute_div(inst),
			I::NEG(inst) => self.execute_neg(inst),
			I::NOT(inst) => self.execute_not(inst),

			I::EQ(inst) => self.execute_eq(inst),
			I::GT(inst) => self.execute_gt(inst),
			I::LT(inst) => self.execute_lt(inst),
			I::GTE(inst) => self.execute_gte(inst),
			I::LTE(inst) => self.execute_lte(inst)
		}?;

		Ok(None)
	}

	#[inline(always)]
	fn execute_push(&mut self, inst: PUSH) -> Result<()> {
		self.stack.push(inst.0);
		Ok(())
	}

	#[inline(always)]
	fn execute_pop(&mut self, _inst: POP) -> Result<Value> {
		self.stack.pop().ok_or(anyhow!("Stack is empty"))
	}

	#[inline(always)]
	fn execute_dup(&mut self, inst: DUP) -> Result<()> {
		let value = self.stack.get(inst.0).cloned().ok_or(anyhow!(
			"Value at index {} does not exist (stack length is {})",
			inst.0,
			self.stack.len()
		))?;

		self.execute_push(PUSH(value))
	}

	#[inline(always)]
	fn execute_load(&mut self, inst: LOAD) -> Result<()> {
		let name = inst.0;
		let value = self
			.bindings
			.get(&name)
			.cloned()
			.ok_or(anyhow!("Binding `{}` does not exist", name))?;

		self.execute_push(PUSH(value))
	}

	#[inline(always)]
	fn execute_store(&mut self, inst: STORE) -> Result<()> {
		let name = inst.0;
		let value = self.execute_pop(POP)?;

		self.bindings.insert(name, value);
		Ok(())
	}

	#[inline(always)]
	fn execute_ret(&mut self, _inst: RET) -> Result<Value> {
		self.execute_pop(POP)
	}

	fn execute_newfunc(&mut self, inst: NEWFUNC) -> Result<()> {
		let arity = inst.0;
		let instructions = inst.1;

		self.execute_push(PUSH(Value::Function {
			arity,
			instructions
		}))
	}

	#[inline(always)]
	fn execute_label(&mut self, inst: LABEL) -> Result<()> {
		let label_name = inst.0.clone();
		self.labels.insert(label_name, inst);

		Ok(())
	}

	fn execute_call(&mut self, _inst: CALL) -> Result<()> {
		let (arity, instructions) = match self.execute_pop(POP)? {
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
			let arg = self.execute_pop(POP)?;
			reversed_args.push(arg);
		}
		self.stack.extend(reversed_args);

		let prev_instructions = std::mem::replace(&mut self.instructions, instructions.into_iter());

		if let Some(val) = self.run()? {
			self.execute_push(PUSH(val))?;
		}

		self.instructions = prev_instructions;
		Ok(())
	}

	#[inline(always)]
	fn execute_jmp(&mut self, inst: JMP) -> Result<Option<Value>> {
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

	#[inline(always)]
	fn execute_jt(&mut self, inst: JT) -> Result<Option<Value>> {
		let condition = match self.execute_pop(POP)? {
			Value::Boolean(v) => v,
			v => bail!("Instruction expected a boolean, found `{v:?}`")
		};

		if !condition {
			return Ok(None);
		}

		self.execute_jmp(JMP(inst.0))
	}

	#[inline(always)]
	fn execute_jtf(&mut self, inst: JTF) -> Result<Option<Value>> {
		let condition = match self.execute_pop(POP)? {
			Value::Boolean(v) => v,
			v => bail!("Instruction expected a boolean, found `{v:?}`")
		};

		match condition {
			true => self.execute_jmp(JMP(inst.0)),
			false => self.execute_jmp(JMP(inst.1))
		}
	}

	#[inline(always)]
	fn execute_add(&mut self, _inst: ADD) -> Result<()> {
		let rhs = self.execute_pop(POP)?;
		let lhs = self.execute_pop(POP)?;

		let value = (lhs + rhs)?;
		self.execute_push(PUSH(value))
	}

	#[inline(always)]
	fn execute_sub(&mut self, _inst: SUB) -> Result<()> {
		let rhs = self.execute_pop(POP)?;
		let lhs = self.execute_pop(POP)?;

		let value = (lhs - rhs)?;
		self.execute_push(PUSH(value))
	}

	#[inline(always)]
	fn execute_mul(&mut self, _inst: MUL) -> Result<()> {
		let rhs = self.execute_pop(POP)?;
		let lhs = self.execute_pop(POP)?;

		let value = (lhs * rhs)?;
		self.execute_push(PUSH(value))
	}

	#[inline(always)]
	fn execute_div(&mut self, _inst: DIV) -> Result<()> {
		let rhs = self.execute_pop(POP)?;
		let lhs = self.execute_pop(POP)?;

		let value = (lhs / rhs)?;
		self.execute_push(PUSH(value))
	}

	#[inline(always)]
	fn execute_neg(&mut self, _inst: NEG) -> Result<()> {
		let operand = self.execute_pop(POP)?;
		let value = (-operand)?;
		self.execute_push(PUSH(value))
	}

	#[inline(always)]
	fn execute_not(&mut self, _inst: NOT) -> Result<()> {
		let operand = self.execute_pop(POP)?;
		let value = !operand;
		self.execute_push(PUSH(value))
	}

	#[inline(always)]
	fn execute_eq(&mut self, _inst: EQ) -> Result<()> {
		let rhs = self.execute_pop(POP)?;
		let lhs = self.execute_pop(POP)?;

		let value = lhs == rhs;
		self.execute_push(PUSH(Value::Boolean(value)))
	}

	#[inline(always)]
	fn execute_gt(&mut self, _inst: GT) -> Result<()> {
		let rhs = self.execute_pop(POP)?;
		let lhs = self.execute_pop(POP)?;

		let value = lhs.gt(&rhs)?;
		self.execute_push(PUSH(value))
	}

	#[inline(always)]
	fn execute_lt(&mut self, _inst: LT) -> Result<()> {
		let rhs = self.execute_pop(POP)?;
		let lhs = self.execute_pop(POP)?;

		let value = lhs.lt(&rhs)?;
		self.execute_push(PUSH(value))
	}

	#[inline(always)]
	fn execute_gte(&mut self, _inst: GTE) -> Result<()> {
		let rhs = self.execute_pop(POP)?;
		let lhs = self.execute_pop(POP)?;

		let value = lhs.gte(&rhs)?;
		self.execute_push(PUSH(value))
	}

	#[inline(always)]
	fn execute_lte(&mut self, _inst: LTE) -> Result<()> {
		let rhs = self.execute_pop(POP)?;
		let lhs = self.execute_pop(POP)?;

		let value = lhs.lte(&rhs)?;
		self.execute_push(PUSH(value))
	}
}
