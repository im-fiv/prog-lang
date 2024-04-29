use anyhow::{Result, bail};
use super::RuntimeValue;

use std::collections::HashMap;
use std::collections::hash_map::Entry;

macro_rules! expect_type {
	(from $args:ident at $index:expr => $kind:ident) => {
		{
			let value = $args.get($index).unwrap().to_owned();

			match value {
				// this limitation can be later fixed if we want to be able to accept intrinsic functions
				RuntimeValue::$kind(inner) => inner,
				_ => bail!("Invalid argument #{} in a function call (expected {}, got {})", $index, stringify!($kind), value.kind())
			}
		}
	};
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeContext {
	pub level: usize,
	pub stdout: String,

	pub con_stdout_allowed: bool,
	pub imports_allowed: bool,

	pub value_table: HashMap<String, RuntimeValue>,
	pub temp_table: Vec<HashMap<String, RuntimeValue>>
}

impl Default for RuntimeContext {
	fn default() -> Self {
		Self::new()
	}
}

impl RuntimeContext {
	pub fn new() -> Self {
		Self {
			level: 0,
			stdout: String::new(),

			con_stdout_allowed: true,
			imports_allowed: true,

			value_table: Self::create_value_table(),
			temp_table: vec![HashMap::new()]
		}
	}

	// define intrinsics here, can be encapsulated into another file later on
	pub fn create_value_table() -> HashMap<String, RuntimeValue> {
		let mut map = HashMap::new();

		map.insert(String::from("print"), RuntimeValue::IntrinsicFunction(print_function, -1));
		fn print_function(context: &mut RuntimeContext, args: Vec<RuntimeValue>) -> Result<RuntimeValue> {
			let to_print = args
				.into_iter()
				.map(|arg| format!("{}", arg))
				.collect::<Vec<String>>()
				.join(" ");

			context.stdout.push_str(&format!("{}\n", to_print)[..]);

			if context.con_stdout_allowed {
				println!("{to_print}");
			}

			Ok(RuntimeValue::Empty)
		}

		// Note: there is no safeguard against cycle imports so the thread's stack will simply overflow
		map.insert(String::from("import"), RuntimeValue::IntrinsicFunction(import_function, 1));
		fn import_function(context: &mut RuntimeContext, args: Vec<RuntimeValue>) -> Result<RuntimeValue> {
			if !context.imports_allowed {
				bail!("Imports in this context are not allowed");
			}

			let path_str = expect_type!(from args at 0 => String);
			let mut path = std::path::Path::new(&path_str).to_path_buf();

			if path.extension().is_none() {
				path.set_extension("prog");
			}

			if !path.exists() {
				bail!("Cannot find the specified file at path '{path_str}'");
			}

			let contents = crate::read_file(path.to_str().unwrap());
			let ast = crate::parse(&contents)?;
			let mut interpreter = crate::Interpreter::new();
			context.clone_into(&mut interpreter.context);
			let result = interpreter.execute(ast)?;
			*context = interpreter.context;

			Ok(result)
		}

		map
	}

	pub fn deeper(&mut self) -> usize {
		self.level += 1;

		if self.temp_table.get(self.level).is_none() {
			self.temp_table.insert(self.level, HashMap::new())
		}

		self.level
	}

	pub fn shallower(&mut self) -> usize {
		self.level -= 1;
		self.temp_table.remove(self.level + 1);

		if self.temp_table.get(self.level).is_none() {
			self.temp_table.insert(self.level, HashMap::new())
		}

		self.level
	}

	pub fn is_temp_write(&self) -> bool {
		self.level > 0
	}

	pub fn key_real(&self, key: &String) -> bool {
		self.value_table.contains_key(key)
	}

	pub fn key_temp(&self, key: &String) -> bool {
		let mut reversed_temp_table = self.temp_table.clone();
		reversed_temp_table.reverse();

		for map in reversed_temp_table {
			if map.contains_key(key) {
				return true;
			}
		}

		false
	}

	pub fn get_value(&self, key: &String) -> Result<RuntimeValue> {
		if !self.key_real(key) && !self.key_temp(key) {
			bail!("Value with name '{key}' does not exist");
		}

		if let Some(value) = self.value_table.get(key) {
			return Ok(value.to_owned());
		}

		let mut reversed_temp_table = self.temp_table.clone();
		reversed_temp_table.reverse();

		for map in reversed_temp_table {
			if let Some(value) = map.get(key) {
				return Ok(value.to_owned());
			}
		}

		unreachable!()
	}

	pub fn insert_value(&mut self, key: String, value: RuntimeValue) -> Result<()> {
		if self.key_real(&key) && self.key_temp(&key) {
			bail!("Value with name '{key}' already exists");
		}

		if self.is_temp_write() {
			let mut map = self.temp_table
				.get(self.level)
				.unwrap()
				.to_owned();

			map.insert(key, value);
			self.temp_table[self.level] = map;
		} else {
			self.value_table.insert(key, value);
		}

		Ok(())
	}

	pub fn update_value(&mut self, key: String, value: RuntimeValue) -> Result<RuntimeValue> {
		if !self.key_real(&key) && !self.key_temp(&key) {
			bail!("Value with name '{key}' does not exist");
		}

		Ok(if self.is_temp_write() && self.key_temp(&key) {
			let mut reversed_temp_table = self.temp_table.clone();
			reversed_temp_table.reverse();

			for (index, mut map) in reversed_temp_table.into_iter().enumerate() {
				if let Entry::Occupied(mut e) = map.entry(key.clone()) {
					let result = Ok(e.insert(value));

					let target_index = self.temp_table.len() - 1 - index;
					self.temp_table[target_index] = map;

					return result;
				}
			}

			None
		} else {
			self.value_table.insert(key, value)
		}.unwrap_or(RuntimeValue::Empty))
	}
}