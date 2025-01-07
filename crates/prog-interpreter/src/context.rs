#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContextFlags {
	pub con_stdout_allowed: bool,
	pub imports_allowed: bool,
	pub inputs_allowed: bool,
	pub externs_allowed: bool
}

impl Default for ContextFlags {
	fn default() -> Self {
		Self {
			con_stdout_allowed: true,
			imports_allowed: true,
			inputs_allowed: true,
			externs_allowed: true
		}
	}
}

#[derive(Debug)]
pub struct Context {
	pub flags: ContextFlags
}

impl Context {
	pub fn new() -> Self {
		Self {
			flags: Default::default()
		}
	}
}

impl Default for Context {
	fn default() -> Self { Self::new() }
}
