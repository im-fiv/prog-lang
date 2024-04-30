use anyhow::{Result, bail};

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::values::RuntimeValue;

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeContext {
	pub level: usize,

	pub stdin: String,
	pub stdout: String,

	pub con_stdout_allowed: bool,
	pub imports_allowed: bool,
	pub input_allowed: bool,

	pub value_table: HashMap<String, RuntimeValue>,
	temp_table: Vec<HashMap<String, RuntimeValue>>
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

			stdin: String::new(),
			stdout: String::new(),

			con_stdout_allowed: true,
			imports_allowed: true,
			input_allowed: true,

			value_table: super::intrinsics::create_value_table(),
			temp_table: vec![HashMap::new()]
		}
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

		let mut reversed_temp_table = self.temp_table.clone();
		reversed_temp_table.reverse();

		for map in reversed_temp_table {
			if let Some(value) = map.get(key) {
				return Ok(value.to_owned());
			}
		}

		if let Some(value) = self.value_table.get(key) {
			return Ok(value.to_owned());
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