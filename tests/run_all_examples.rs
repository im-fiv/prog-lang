use std::ffi::OsStr;
use std::fs::ReadDir;
use std::path::Path;

use anyhow::Result;

fn execute_string(source: &str, file: &str) -> Result<()> {
	let ts = prog_lexer::lex(source, file)?;
	let tokens = ts.buffer();

	let ps = prog_parser::ParseStream::new(tokens);
	let ast = ps.parse::<prog_parser::ast::Program>()?;

	let mut interpreter = prog_interpreter::Interpreter::new(ast);
	interpreter.context_mut().flags.con_stdout_allowed = false;
	interpreter.interpret()?;

	Ok(())
}

fn iterate_dir(paths: ReadDir, exclusions: &[String]) {
	let file_extension = OsStr::new("prog");

	for path in paths {
		let path = path.expect("Failed to resolve path").path();

		if path.is_dir() {
			let paths = path.read_dir().expect("Failed to read directory");

			iterate_dir(paths, exclusions);
			continue;
		}

		if path.extension().unwrap_or(OsStr::new("")) != file_extension {
			continue;
		}

		if exclusions.iter().any(|exclusion| {
			let exclusion_path = Path::new(exclusion);
			path.ends_with(exclusion_path)
		}) {
			println!("Skipping file {}", path.display());
			continue;
		}

		println!("Running file {}", path.display());

		let path_str = path.to_str().unwrap();

		let source = prog_utils::read_file(path_str);
		let execution_result = execute_string(&source, path_str);

		assert!(
			execution_result.is_ok(),
			"Execution failed: {}",
			execution_result.unwrap_err()
		);
	}
}

#[test]
fn run_all_examples() {
	let paths = std::fs::read_dir("./examples").expect("Failed to read directory");
	let exclusions = vec![String::from("mandelbrot_set.prog")];

	iterate_dir(paths, exclusions.as_slice());
}
