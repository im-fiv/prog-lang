pub mod cli;
mod error;

use cli::Cli;
use error::ProgError;

use clap::Parser;
use prog_interpreter::Interpreter;
use prog_utils::read_file;

#[cfg(feature = "api")]
const NAME_STDIN: &str = "<STDIN>";
#[cfg(feature = "repl")]
const NAME_REPL: &str = "<REPL>";

#[cfg(feature = "api")]
fn serialize_prog(err: ProgError) -> Result<String, String> {
	macro_rules! ser {
		($err:ident) => {
			serde_json::to_string_pretty($err).map_err(|e| e.to_string())
		};
	}

	match err {
		ProgError::Lex(ref e) => ser!(e),
		ProgError::Parse(ref e) => ser!(e),
		ProgError::Interpret(ref e) => ser!(e)
	}
}

fn execute_run_command(args: cli::RunCommand) {
	use prog_interpreter::ValueKind;

	let contents = read_file(&args.file_path);
	let ts = match prog_lexer::lex(&contents, &args.file_path).map_err(ProgError::Lex) {
		Ok(ts) => ts,
		Err(err) => {
			eprintln!("{err}");
			return;
		}
	};

	let ps = prog_parser::ParseStream::new(ts.buffer());
	let ast = match ps
		.parse::<prog_parser::ast::Program>()
		.map_err(ProgError::Parse)
	{
		Ok(ast) => ast,
		Err(err) => {
			eprintln!("{err}");
			return;
		}
	};

	let mut interpreter = Interpreter::new(ast);
	match interpreter.interpret().map_err(ProgError::Interpret) {
		Ok(val) if !matches!(val.kind(), ValueKind::Empty) => println!("{val}"),
		Err(err) => eprintln!("{err}"),

		_ => ()
	};
}

#[cfg(feature = "repl")]
fn repl() {
	// TODO
	todo!("REPL")

	/*
	use rustyline::error::ReadlineError;
	use rustyline::DefaultEditor;

	let mut rl = DefaultEditor::new().unwrap();
	let mut interpreter = None;
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
				let ts = match prog_lexer::lex(&source, NAME_REPL).map_err(ProgError::Lex) {
					Ok(ts) => ts,
					Err(err) => {
						eprintln!("{err}");
						continue;
					}
				};

				let ps = prog_parser::ParseStream::new(ts.buffer());
				let ast = match ps.parse::<prog_parser::ast::Program>().map_err(ProgError::Parse) {
					Ok(ast) => ast,
					Err(err) => {
						eprintln!("{err}");
						continue;
					}
				};

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
	*/
}

#[cfg(feature = "api")]
fn execute_serve_command(args: cli::ServeCommand) {
	use actix_cors::Cors;
	use actix_web::middleware::Logger;
	use actix_web::{post, App, HttpResponse, HttpServer, Responder};
	use cfg_if::cfg_if;
	use prog_interpreter::Value;

	env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

	fn handle_prog_error(error: ProgError) -> HttpResponse {
		let json = match serialize_prog(error) {
			Ok(s) => s,
			Err(error) => return HttpResponse::InternalServerError().body(error)
		};

		HttpResponse::Ok().body(json)
	}

	#[post("/execute")]
	async fn execute_str(req_body: String) -> impl Responder {
		let ts = match prog_lexer::lex(&req_body, NAME_STDIN).map_err(ProgError::Lex) {
			Ok(ts) => ts,
			Err(err) => return handle_prog_error(err)
		};

		let ps = prog_parser::ParseStream::new(ts.buffer());
		let ast = match ps
			.parse::<prog_parser::ast::Program>()
			.map_err(ProgError::Parse)
		{
			Ok(ast) => ast,
			Err(err) => return handle_prog_error(err)
		};

		let mut interpreter = Interpreter::new(ast);
		interpreter.context_mut().flags.con_stdout_allowed = false;
		interpreter.context_mut().flags.imports_allowed = false;
		interpreter.context_mut().flags.inputs_allowed = false;

		let result = match interpreter.interpret().map_err(ProgError::Interpret) {
			Ok(val) => val,
			Err(err) => return handle_prog_error(err)
		};

		#[derive(Debug, serde::Serialize)]
		struct Result {
			value: Value,
			stdin: Vec<u8>,
			stdout: Vec<u8>
		}

		let stdin = interpreter.context().stdin.clone();
		let stdout = interpreter.context().stdout.clone();

		let result_struct = Result {
			value: result,
			stdin,
			stdout
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
