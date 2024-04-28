pub mod interpreter;
pub mod cli;

use interpreter::Interpreter;
use cli::Cli;

pub mod parser;
use parser::parse;

use actix_web::{post, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;
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

fn execute_run_command(args: cli::RunCommand) {
	let contents = read_file(&args.file_path);
	let ast = parse(&contents).unwrap();
	let mut interpreter = Interpreter::new();
	let result = interpreter.execute(ast).unwrap();

	dbg!(result);
}

fn execute_serve_command(args: cli::ServeCommand) {
	fn handle_anyhow_error(error: anyhow::Error) -> HttpResponse {
		let as_serde_error = serde_error::Error::new(&*error);
		let json = match serde_json::to_string_pretty(&as_serde_error) {
			Ok(json) => json,
			Err(error) => return HttpResponse::from_error(error)
		};

		HttpResponse::Ok().body(json)
	}

	#[post("/execute")]
	async fn execute_str(req_body: String) -> impl Responder {
		println!("New request with body: {req_body}");

		let ast = match parse(&req_body) {
			Ok(ast) => ast,
			Err(error) => return handle_anyhow_error(error)
		};

		let mut interpreter = Interpreter::new();
		interpreter.context.no_con_stdout = true;

		let result = match interpreter.execute(ast) {
			Ok(result) => result,
			Err(error) => return handle_anyhow_error(error)
		};

		#[derive(Debug, Serialize)]
		struct Result {
			pub value: interpreter::values::RuntimeValue,
			pub stdout: String
		}

		let result_struct = Result {
			value: result,
			stdout: interpreter.context.stdout
		};

		let json = match serde_json::to_string_pretty(&result_struct) {
			Ok(json) => json,
			Err(error) => return HttpResponse::from_error(error)
		};

		HttpResponse::Ok().body(json)
	}
	
	#[actix_web::main]
	async fn run_server(port: u16) -> std::io::Result<()> {
		HttpServer::new(|| {
			App::new()
				.service(execute_str)
		})
		.bind(("127.0.0.1", port))?
		.run()
		.await
	}

	run_server(args.port).unwrap();
}

fn main() {
	let args = Cli::parse();

	match args.subcommand {
		cli::CLISubcommand::Run(command) => execute_run_command(command),
		cli::CLISubcommand::Serve(command) => execute_serve_command(command)
	}
}