use anyhow::{Result, bail};
use std::collections::HashMap;

use super::RuntimeValue;

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeContext {
	pub write_to_temp: bool,

	pub value_table: HashMap<String, RuntimeValue>,
	pub temp_table: HashMap<String, RuntimeValue>
}

impl Default for RuntimeContext {
	fn default() -> Self {
		Self::new()
	}
}

impl RuntimeContext {
	pub fn new() -> Self {
		Self {
			write_to_temp: false,

			value_table: Self::create_value_table(),
			temp_table: HashMap::new()
		}
	}

	// define intrinsics here, can be encapsulated into another file later on
	pub fn create_value_table() -> HashMap<String, RuntimeValue> {
		let mut map = HashMap::new();

		map.insert(String::from("print"), RuntimeValue::IntrinsicFunction(print_function));
		fn print_function(_context: &mut RuntimeContext, args: Vec<RuntimeValue>) -> Result<RuntimeValue> {
			let to_print = args
				.into_iter()
				.map(|arg| format!("{}", arg))
				.collect::<Vec<String>>()
				.join(" ");

			println!("{to_print}");
			Ok(RuntimeValue::Empty)
		}

		map
	}

	pub fn set_write_to_temp(&mut self, value: bool) {
		self.write_to_temp = value;
		self.temp_table.clear();
	}

	pub fn key_real(&self, key: &String) -> bool {
		self.value_table.contains_key(key)
	}

	pub fn key_temp(&self, key: &String) -> bool {
		self.temp_table.contains_key(key)
	}

	pub fn get_value(&self, key: &String) -> Result<RuntimeValue> {
		if !self.key_real(key) && !self.key_temp(key) {
			bail!("Value with name '{key}' does not exist");
		}

		Ok(self.temp_table.get(key)
			.or(self.value_table.get(key))
			.unwrap()
			.to_owned())
	}

	pub fn insert_value(&mut self, key: String, value: RuntimeValue) -> Result<()> {
		if self.key_real(&key) && self.key_temp(&key) {
			bail!("Value with name '{key}' already exists");
		}

		if self.write_to_temp {
			self.temp_table.insert(key, value);
		} else {
			self.value_table.insert(key, value);
		}

		Ok(())
	}

	pub fn update_value(&mut self, key: String, value: RuntimeValue) -> Result<RuntimeValue> {
		if !self.key_real(&key) && !self.key_temp(&key) {
			bail!("Value with name '{key}' does not exist");
		}

		Ok(if self.write_to_temp && self.temp_table.contains_key(&key) {
			self.temp_table.insert(key, value)
		} else {
			self.value_table.insert(key, value)
		}.unwrap_or(RuntimeValue::Empty))
	}
}