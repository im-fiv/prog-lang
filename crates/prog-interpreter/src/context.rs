use anyhow::{Result, bail};

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::cell::RefCell;

use crate::RuntimeValue;

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeContext {
	pub level: usize,

	pub stdin: String,
	pub stdout: String,

	pub con_stdout_allowed: bool,
	pub imports_allowed: bool,
	pub input_allowed: bool,

	pub value_table: HashMap<String, RuntimeValue>,
	temp_table: HashMap<usize, RefCell<HashMap<String, RuntimeValue>>>
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
			temp_table: HashMap::from([
				(0, RefCell::new(HashMap::new()))
			])
		}
	}

	pub fn deeper(&mut self) -> usize {
		self.level += 1;

		self
			.temp_table
			.entry(self.level)
			.or_insert_with(||
				RefCell::new(HashMap::new())
			);

		self.level
	}

	pub fn shallower(&mut self) -> usize {
		self.level -= 1;
		self.temp_table.remove(&(self.level + 1));

		self
			.temp_table
			.entry(self.level)
			.or_insert_with(||
				RefCell::new(HashMap::new())
			);

		self.level
	}

	pub fn is_temp_write(&self) -> bool {
		self.level > 0
	}

	pub fn key_real(&self, key: &String) -> bool {
		self.value_table.contains_key(key)
	}

	pub fn key_temp(&self, key: &String) -> bool {
		self
			.temp_table
			.values()
			.any(|map|
				map.borrow().contains_key(key)
			)
	}

	pub fn get_value(&self, key: &String) -> Result<RuntimeValue> {
		if !self.key_real(key) && !self.key_temp(key) {
			bail!("Value with name '{key}' does not exist");
		}

		// Iterating through temp table maps in reverse order
		for map_index in (0..self.temp_table.len()).rev() {
			// Getting a reference to the RefCell of the map
			let map = self
				.temp_table
				.get(&map_index)
				.unwrap_or_else(|| unreachable!("Temp table map at index `{map_index}` does not exist"));

			// Getting the value from it
			if let Some(value) = map.borrow().get(key) {
				return Ok(value.to_owned());
			}
		}

		if let Some(value) = self.value_table.get(key) {
			return Ok(value.to_owned());
		}

		unreachable!()
	}

	pub fn get_value_mut(&mut self, key: &String) -> Result<&mut RuntimeValue> {
		if !self.key_real(key) && !self.key_temp(key) {
			bail!("Value with name '{key}' does not exist");
		}
		
		// Iterating through temp table maps in reverse order
		for map_index in (0..self.temp_table.len()).rev() {
			// Getting a mutable reference to the RefCell of the map
			let map = self
				.temp_table
				.get_mut(&map_index)
				.unwrap_or_else(|| unreachable!("Temp table map at index `{map_index}` does not exist"));

			// Getting a mutable reference to the map itself
			let mut borrowed_map = map.borrow_mut();

			// Getting the value from it
			if let Some(value) = borrowed_map.get_mut(key) {
				// Warning: dark magic ahead!
				// Extending the lifetime of the mutable borrow
				return Ok(unsafe {
					&mut *(value as *mut _)
				});
			}
		}

		if let Some(value) = self.value_table.get_mut(key) {
			return Ok(value);
		}

		unreachable!()
	}

	pub fn insert_value(&mut self, key: String, value: RuntimeValue) -> Result<()> {
		if self.key_real(&key) && self.key_temp(&key) {
			bail!("Value with name '{key}' already exists");
		}

		// Determining which table to write to
		if self.is_temp_write() {
			// Getting a mutable reference to the RefCell of the map
			let map = self
				.temp_table
				.get_mut(&self.level)
				.unwrap_or_else(|| unreachable!("Temp table map at index `{}` does not exist", self.level));

			map.borrow_mut().insert(key, value);
		} else {
			self.value_table.insert(key, value);
		}

		Ok(())
	}

	pub fn update_value(&mut self, key: String, value: RuntimeValue) -> Result<RuntimeValue> {
		if !self.key_real(&key) && !self.key_temp(&key) {
			bail!("Value with name '{key}' does not exist");
		}

		// Determining which table to write to
		let old_value = if self.is_temp_write() && self.key_temp(&key) {
			// Iterating through temp table maps in reverse order
			for map_index in (0..self.temp_table.len()).rev() {
				// Getting a mutable reference to the RefCell of the map
				let map = self
					.temp_table
					.get_mut(&map_index)
					.unwrap_or_else(|| unreachable!("Temp table map at index `{map_index}` does not exist"));

				// Getting a mutable reference to the map itself
				let mut borrowed_map = map.borrow_mut();

				// Getting an entry from the mutable map reference
				if let Entry::Occupied(mut e) = borrowed_map.entry(key.clone()) {
					// Updating the entry
					let result = Ok(e.insert(value));

					return result;
				}
			}

			None
		} else {
			self.value_table.insert(key, value)
		};

		Ok(old_value.unwrap_or(RuntimeValue::Empty))
	}
}