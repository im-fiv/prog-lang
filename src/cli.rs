use clap::Parser;

pub const DEFAULT_INPUT_FP: &str = "input.prog";

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Cli {
	#[arg(default_value = DEFAULT_INPUT_FP)]
	pub file_path: String
}