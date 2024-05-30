pub mod cli;

use cli::Cli;
use prog_parser::Parser as ProgParser;
use prog_interpreter::{Interpreter, RuntimeValue};
use prog_utils::read_file;

use actix_web::{App, HttpResponse, HttpServer, Responder, post};
use actix_cors::Cors;

use serde::Serialize;
use clap::Parser;

fn serialize_anyhow(anyhow_error: anyhow::Error) -> Result<String, String> {
	let interpret_error = anyhow_error
		.downcast_ref::<
			prog_interpreter::InterpretError
		>();

	let parse_error = anyhow_error
		.downcast_ref::<
			prog_parser::ParseError
		>();

	if let Some(interpret_error) = interpret_error {
		return serde_json
			::to_string_pretty(interpret_error)
			.map_err(|e| e.to_string());
	}

	if let Some(parse_error) = parse_error {
		return serde_json
			::to_string_pretty(parse_error)
			.map_err(|e| e.to_string());
	}

	Err(String::from("Failed to serialize anyhow error to JSON"))
}

fn execute_run_command(args: cli::RunCommand) {
	let contents = read_file(&args.file_path);

	let parser = ProgParser::new(&contents, &args.file_path);
	let ast = parser.parse().unwrap();

	let mut interpreter = Interpreter::new(contents, args.file_path);

	let result = interpreter.execute(ast, false).unwrap();

	dbg!(result);
}

fn execute_serve_command(args: cli::ServeCommand) {
	fn handle_anyhow_error(error: anyhow::Error) -> HttpResponse {
		let json = match serialize_anyhow(error) {
			Ok(s) => s,
			Err(error) => return HttpResponse::InternalServerError().body(error)
		};

		HttpResponse::Ok().body(json)
	}

	#[post("/execute")]
	async fn execute_str(req_body: String) -> impl Responder {
		println!("New request with body: {req_body}");

		let parser = ProgParser::new(&req_body, "stdin");
		let ast = match parser.parse() {
			Ok(ast) => ast,
			Err(error) => return handle_anyhow_error(error)
		};

		let mut interpreter = Interpreter::new(req_body, String::from("stdin"));
		interpreter.context.flags.con_stdout_allowed = false;
		interpreter.context.flags.imports_allowed = false;
		interpreter.context.flags.inputs_allowed = false;

		let result = match interpreter.execute(ast, false) {
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