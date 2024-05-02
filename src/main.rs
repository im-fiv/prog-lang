pub mod cli;

use cli::Cli;
use prog_parser::Parser as ProgParser;
use prog_interpreter::{Interpreter, RuntimeValue};
use prog_utils::read_file;

use actix_web::{App, HttpResponse, HttpServer, Responder, post};
use actix_cors::Cors;

use serde::Serialize;
use clap::Parser;

fn execute_run_command(args: cli::RunCommand) {
	let contents = read_file(args.file_path);

	// TODO: extract filename
	let parser = ProgParser::new(&contents, String::from("TODO"));
	let ast = parser.parse().unwrap();

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

		let parser = ProgParser::new(&req_body, String::from("stdin"));
		let ast = match parser.parse() {
			Ok(ast) => ast,
			Err(error) => return handle_anyhow_error(error)
		};

		let mut interpreter = Interpreter::new();
		interpreter.context.con_stdout_allowed = false;
		interpreter.context.imports_allowed = false;
		interpreter.context.input_allowed = false;

		let result = match interpreter.execute(ast) {
			Ok(result) => result,
			Err(error) => return handle_anyhow_error(error)
		};

		#[derive(Debug, Serialize)]
		struct Result {
			value: RuntimeValue,
			stdin: String,
			stdout: String
		}

		let result_struct = Result {
			value: result,
			stdin: interpreter.context.stdin,
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
			let cors = Cors::permissive();

			App::new()
				.wrap(cors)
				.service(execute_str)
				.service(actix_files::Files::new("/", "./website").show_files_listing())
		})
		.bind(("0.0.0.0", port))?
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