use clap::{Parser, Subcommand, Args};

pub const DEFAULT_INPUT_FP: &str = "input.prog";
pub const DEFAULT_SERVER_PORT: u16 = 8080;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Cli {
	/// Action
	#[clap(subcommand)]
	pub subcommand: CLISubcommand
}

#[derive(Debug, Subcommand)]
pub enum CLISubcommand {
	/// Interpret a file
	Run(RunCommand),

	/// Launch REST server
	Serve(ServeCommand)
}

#[derive(Debug, Args)]
pub struct RunCommand {
	#[arg(default_value = DEFAULT_INPUT_FP)]
	/// Target file path
	pub file_path: String
}

#[derive(Debug, Args)]
pub struct ServeCommand {
	#[arg(default_value_t = DEFAULT_SERVER_PORT)]
	/// Target port for the running server
	pub port: u16
}