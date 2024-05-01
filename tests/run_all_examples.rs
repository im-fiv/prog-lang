use std::ffi::OsStr;
use std::fs::ReadDir;

use anyhow::Result;

fn execute_string(code: String) -> Result<prog_interpreter::RuntimeValue> {
	let ast = prog_parser::parse(&code)?;

	let mut interpreter = prog_interpreter::Interpreter::new();
	interpreter.context.con_stdout_allowed = false;

	let result = interpreter.execute(ast)?;
	
	Ok(result)
}

fn iterate_dir(paths: ReadDir) {
	let file_extension = OsStr::new("prog");

	for path in paths {
		let path = path
			.expect("Failed to resolve path")
			.path();

		if path.is_dir() {
			let paths = path
				.read_dir()
				.expect("Failed to read directory");

			iterate_dir(paths);
			continue;
		}

		if path.extension().unwrap_or(OsStr::new("")) != file_extension {
			continue;
		}

		println!("Running file {}", path.display());

		let contents = prog_utils::read_file(path);
		let execution_result = execute_string(contents);

		assert!(execution_result.is_ok(), "Execution failed: {}", execution_result.unwrap_err())
	}
}

#[test]
fn run_all_examples() {
	let paths = std::fs::read_dir("./examples").expect("Failed to read directory");
	iterate_dir(paths);
}