use anyhow::{Result, bail};
use std::collections::HashMap;

use super::RuntimeValue;

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeContext {
	variable_table: HashMap<String, RuntimeValue>
}

impl Default for RuntimeContext {
	fn default() -> Self {
		Self::new()
	}
}

impl RuntimeContext {
	pub fn new() -> Self {
		Self {
			variable_table: HashMap::new()
		}
	}

	pub fn key_exists(&self, key: &String) -> bool {
		self.variable_table.contains_key(key)
	}

	pub fn get_value(&self, key: &String) -> Result<RuntimeValue> {
		if !self.key_exists(key) {
			bail!("Value with name '{key}' does not exist")
		}

		Ok(self.variable_table.get(key).unwrap().to_owned())
	}

	pub fn insert_value(&mut self, key: String, value: RuntimeValue) -> Result<()> {
		if self.key_exists(&key) {
			bail!("Value with name '{key}' already exists");
		}

		self.variable_table.insert(key, value);
		Ok(())
	}

	pub fn delete_value(&mut self, key: &String) -> Result<RuntimeValue> {
		if !self.key_exists(key) {
			bail!("Value with name '{key}' does not exist")
		}

		Ok(self.variable_table.remove(key).unwrap_or_else(|| unreachable!()))
	}

	pub fn update_value(&mut self, key: String, value: RuntimeValue) -> Result<RuntimeValue> {
		if !self.key_exists(&key) {
			bail!("Value with name '{key}' does not exist")
		}

		Ok(self.variable_table.insert(key, value).unwrap())
	}
}