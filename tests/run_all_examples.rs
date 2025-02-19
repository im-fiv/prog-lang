use std::ffi::OsStr;
use std::fs::ReadDir;
use std::path::Path;

use prog_lang::ProgResult;

#[allow(unused_variables)]
fn execute_string<'src>(source: &'src str, file: &'src str) -> ProgResult<'src, ()> {
	let ts = prog_lexer::lex(source, file)?;

	let ps = prog_parser::ParseStream::new(&ts);
	let ast = ps.parse::<prog_parser::ast::Program>()?;

	let mut interpreter = prog_interpreter::Interpreter::new();
	interpreter.context.inner_mut().flags.con_stdout_allowed = false;
	interpreter.evaluate(ast)?;

	Ok(())
}

fn iterate_dir(paths: ReadDir, exclusions: &[&str]) {
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

// #[test]
// fn run_all_examples() {
// 	let paths = std::fs::read_dir("./examples").expect("Failed to read directory");
// 	let exclusions = vec!["mandelbrot_set.prog"];
//
// 	iterate_dir(paths, exclusions.as_slice());
// }

#[test]
fn run_all_cases() {
	let paths = std::fs::read_dir("./tests/cases").expect("Failed to read directory");
	iterate_dir(paths, &[]);
}
