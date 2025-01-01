pub mod cli;

use clap::Parser;
use cli::Cli;
use prog_interpreter::Interpreter;
use prog_parser::Parser as ProgParser;
use prog_utils::read_file;

#[cfg(feature = "api")]
const NAME_STDIN: &str = "<STDIN>";
#[cfg(feature = "repl")]
const NAME_REPL: &str = "<REPL>";

#[cfg(feature = "api")]
fn serialize_anyhow(anyhow_error: anyhow::Error) -> Result<String, String> {
	let interpret_error = anyhow_error.downcast_ref::<prog_interpreter::InterpretError>();

	let parse_error = anyhow_error.downcast_ref::<prog_parser::ParseError>();

	if let Some(interpret_error) = interpret_error {
		return serde_json::to_string_pretty(interpret_error).map_err(|e| e.to_string());
	}

	if let Some(parse_error) = parse_error {
		return serde_json::to_string_pretty(parse_error).map_err(|e| e.to_string());
	}

	Err(String::from("Failed to serialize anyhow error to JSON"))
}

fn execute_run_command(args: cli::RunCommand) {
	use prog_interpreter::ValueKind;

	let contents = read_file(&args.file_path);

	// TODO
	// let token_stream = prog_lexer::lex(&contents, &args.file_path).unwrap();

	let parser = ProgParser::new(&contents, &args.file_path);
	let ast = parser.parse().unwrap();

	let mut interpreter = Interpreter::new();
	let result = interpreter.interpret(contents, args.file_path, ast, false);

	match result {
		Ok(r) if !matches!(r.kind(), ValueKind::Empty) => println!("{r}"),
		Err(e) => eprintln!("{e}"),

		_ => ()
	};
}

#[cfg(feature = "repl")]
fn repl() {
	use rustyline::error::ReadlineError;
	use rustyline::DefaultEditor;

	let mut rl = DefaultEditor::new().unwrap();
	let mut interpreter = Interpreter::new();
	let mut line_counter = 0usize;

	println!(
		"{} REPL v{}",
		env!("CARGO_PKG_NAME"),
		env!("CARGO_PKG_VERSION")
	);

	loop {
		let line = rl.readline("> ");

		match line {
			Ok(line) => {
				rl.add_history_entry(line.as_str()).unwrap();

				// A hacky way to keep track of the line number without having to re-interpret the entire REPL
				let source = format!("{}{}", "\n".repeat(line_counter), line);

				let parser = ProgParser::new(&source, NAME_REPL);
				let ast = parser.parse();

				if ast.is_err() {
					println!("{:?}", ast.unwrap_err());
					continue;
				}

				let result = interpreter.interpret(source, NAME_REPL, ast.unwrap(), false);

				if result.is_err() {
					println!("{:?}", result.unwrap_err());
					continue;
				}

				println!("{}", result.unwrap());
				line_counter += 1;
			}

			Err(ReadlineError::Interrupted) => {
				println!("CTRL-C");
				break;
			}

			Err(ReadlineError::Eof) => {
				println!("CTRL-D");
				break;
			}

			Err(err) => {
				println!("Error: {err:?}");
				break;
			}
		}
	}
}

#[cfg(feature = "api")]
fn execute_serve_command(args: cli::ServeCommand) {
	use actix_cors::Cors;
	use actix_web::middleware::Logger;
	use actix_web::{post, App, HttpResponse, HttpServer, Responder};
	use cfg_if::cfg_if;
	use prog_interpreter::Value;

	env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

	fn handle_anyhow_error(error: anyhow::Error) -> HttpResponse {
		let json = match serialize_anyhow(error) {
			Ok(s) => s,
			Err(error) => return HttpResponse::InternalServerError().body(error)
		};

		HttpResponse::Ok().body(json)
	}

	#[post("/execute")]
	async fn execute_str(req_body: String) -> impl Responder {
		let parser = ProgParser::new(&req_body, NAME_STDIN);
		let ast = match parser.parse() {
			Ok(ast) => ast,
			Err(error) => return handle_anyhow_error(error)
		};

		let mut interpreter = Interpreter::new();
		interpreter.context.deref_mut().flags.con_stdout_allowed = false;
		interpreter.context.deref_mut().flags.imports_allowed = false;
		interpreter.context.deref_mut().flags.inputs_allowed = false;

		let result = match interpreter.interpret(req_body, NAME_STDIN, ast, false) {
			Ok(result) => result,
			Err(error) => return handle_anyhow_error(error)
		};

		#[derive(Debug, serde::Serialize)]
		struct Result {
			value: Value,
			stdin: String,
			stdout: String
		}

		let unwrapped_context = interpreter.context.unwrap_or_clone();
		let result_struct = Result {
			value: result,
			stdin: unwrapped_context.stdin,
			stdout: unwrapped_context.stdout
		};

		let json = match serde_json::to_string_pretty(&result_struct) {
			Ok(json) => json,
			Err(error) => return HttpResponse::from_error(error)
		};

		HttpResponse::Ok().body(json)
	}

	#[actix_web::main]
	async fn run_server(port: u16) -> std::io::Result<()> {
		let server = HttpServer::new(|| {
			let app = App::new()
				.wrap(Logger::default())
				.wrap(Cors::permissive())
				.service(execute_str);

			cfg_if! {
				if #[cfg(feature = "website")] {
					app.service(actix_files::Files::new("/", "./website").show_files_listing())
				} else {
					app
				}
			}
		});

		server.bind(("0.0.0.0", port))?.run().await
	}

	run_server(args.port).unwrap();
}

fn main() {
	let Cli { subcommand } = Cli::parse();

	#[cfg(feature = "repl")]
	let subcommand = {
		if subcommand.is_none() {
			return repl();
		}

		subcommand.unwrap()
	};

	match subcommand {
		cli::CLISubcommand::Run(command) => execute_run_command(command),

		#[cfg(feature = "api")]
		cli::CLISubcommand::Serve(command) => execute_serve_command(command)
	}
}
