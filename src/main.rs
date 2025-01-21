pub mod cli;
mod error;

use cli::Cli;
use error::ProgError;

use clap::Parser;
use prog_interpreter::{Interpreter, ValueKind};
use prog_utils::read_file;

fn evaluate_file(file_path: String, _debug: bool) {
	use prog_parser::ast;

	let contents = read_file(&file_path);
	let ts = match prog_lexer::lex(&contents, &file_path).map_err(ProgError::Lex) {
		Ok(ts) => ts,
		Err(err) => {
			eprintln!("{err}");
			return;
		}
	};

	let ps = prog_parser::ParseStream::new(&ts);
	let parse_result = ps.parse::<ast::Program>().map_err(ProgError::Parse);
	let ast = match parse_result {
		Ok(ast) => ast,
		Err(err) => {
			eprintln!("{err}");
			return;
		}
	};

	let mut interpreter = Interpreter::new();
	match interpreter.evaluate(ast).map_err(ProgError::Interpret) {
		Ok(val) if val.kind() != ValueKind::None => println!("{val}"),
		Err(err) => eprintln!("{err}"),

		_ => ()
	};
}

fn main() {
	let Cli { file_path, debug } = Cli::parse();
	evaluate_file(file_path, debug);
}
