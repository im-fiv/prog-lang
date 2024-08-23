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

#[cfg(feature = "vm")]
fn execute_bytecode(bytes: &[u8]) {
	use prog_vm::{Bytecode, VM};

	let bytecode = Bytecode::from_bytes(&bytes).unwrap();
	let mut vm = VM::new(bytecode).unwrap();

	match vm.run() {
		Ok(v) => {
			if let Some(v) = v {
				println!("{v}")
			}
		}

		Err(e) => eprintln!("{e}")
	}
}

fn execute_run_command(args: cli::RunCommand) {
	use prog_interpreter::ValueKind;

	let file_extension = std::path::Path::new(&args.file_path)
		.extension()
		.and_then(std::ffi::OsStr::to_str)
		.unwrap();

	if file_extension == "progc" {
		#[cfg(not(feature = "vm"))]
		panic!("Running bytecode files is not supported due to inactive `vm` feature");

		#[cfg(feature = "vm")]
		{
			let bytecode = std::fs::read(&args.file_path).unwrap();
			return execute_bytecode(&bytecode);
		}
	}

	let contents = read_file(&args.file_path);

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

#[cfg(feature = "vm")]
fn execute_compile_command(args: cli::CompileCommand) {
	use prog_compiler::Compiler;

	let contents = read_file(&args.file_path);

	let parser = ProgParser::new(&contents, &args.file_path);
	let ast = parser.parse().unwrap();

	let mut compiler = Compiler::new();
	let bytecode = compiler.compile(ast);

	if let Err(e) = bytecode {
		eprintln!("{e}");
		return;
	}

	// TODO: allow customization of output file paths
	let bytecode = bytecode.unwrap();
	let serialized = bytecode.as_bytes().unwrap();
	let human_readable = format!("{bytecode}");

	std::fs::write(cli::DEFAULT_OUTPUT_BC_FP, &serialized).unwrap();
	std::fs::write(cli::DEFAULT_OUTPUT_BC_FMT_FP, human_readable).unwrap();

	if args.run {
		execute_bytecode(&serialized);
	}
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
		interpreter.context.flags.con_stdout_allowed = false;
		interpreter.context.flags.imports_allowed = false;
		interpreter.context.flags.inputs_allowed = false;

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

		#[cfg(feature = "vm")]
		cli::CLISubcommand::Compile(command) => execute_compile_command(command),

		#[cfg(feature = "api")]
		cli::CLISubcommand::Serve(command) => execute_serve_command(command)
	}
}
