pub mod interpreter;
pub mod cli;

use interpreter::Interpreter;
use cli::Cli;

pub mod parser;
use parser::parse;

use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use clap::Parser;

pub fn read_file(path: &str) -> String {
	let file = File::open(path)
		.unwrap_or_else(|_| panic!("Failed to open file `{}` (read)", path));
	
	let mut reader = BufReader::new(file);
	let mut contents = String::new();
	
	reader
		.read_to_string(&mut contents)
		.unwrap_or_else(|_| panic!("Failed to read from file `{}`", path));

	contents.replace("\r\n", "\n")
}

fn main() {
	let args = Cli::parse();

	let contents = read_file(&args.file_path);
	let ast = parse(&contents).unwrap();
	let mut interpreter = Interpreter::new();
	let result = interpreter.execute(ast).unwrap();

	dbg!(result);
}