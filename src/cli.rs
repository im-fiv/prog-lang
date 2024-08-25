use clap::{Args, Parser, Subcommand};

/// Default input file path
pub const DEFAULT_INPUT_FP: &str = "input.prog";
/// Default bytecode output file path
pub const DEFAULT_OUTPUT_BC_FP: &str = "output.progc";
/// Default human-readable bytecode output file path
pub const DEFAULT_OUTPUT_BC_FMT_FP: &str = "output.progc.txt";
/// Default server serving port
pub const DEFAULT_SERVER_PORT: u16 = 80;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Cli {
	#[cfg(not(feature = "repl"))]
	#[clap(subcommand)]
	/// Action
	pub subcommand: CLISubcommand,

	#[cfg(feature = "repl")]
	#[clap(subcommand)]
	/// Action
	pub subcommand: Option<CLISubcommand>
}

#[derive(Debug, Subcommand)]
pub enum CLISubcommand {
	/// Interpret a file
	Run(RunCommand),

	#[cfg(feature = "vm")]
	/// Compile a file to bytecode
	Compile(CompileCommand),

	#[cfg(feature = "api")]
	/// Launch REST server
	Serve(ServeCommand)
}

#[derive(Debug, Args)]
pub struct RunCommand {
	#[arg(default_value = DEFAULT_INPUT_FP)]
	/// Target file path
	pub file_path: String,

	#[arg(long, short, default_value_t = false)]
	/// Enable debug information
	pub debug: bool
}

#[derive(Debug, Args)]
pub struct CompileCommand {
	#[arg(default_value = DEFAULT_INPUT_FP)]
	/// Target file path
	pub file_path: String,

	// TODO: allow customization of output file paths
	#[arg(long, short, default_value_t = false)]
	/// Run the file after compilation
	pub run: bool,

	#[arg(long, short, default_value_t = false, requires = "run")]
	/// Enable debug information
	pub debug: bool
}

#[cfg(feature = "api")]
#[derive(Debug, Args)]
pub struct ServeCommand {
	#[arg(default_value_t = DEFAULT_SERVER_PORT)]
	/// Target port for the running server
	pub port: u16
}
