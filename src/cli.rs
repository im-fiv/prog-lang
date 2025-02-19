use clap::Parser;

/// Default input file path
#[cfg(debug_assertions)]
pub const DEFAULT_INPUT_FP: &str = "input.prog";

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Cli {
	/// Target file path
	#[cfg_attr(debug_assertions, arg(default_value = DEFAULT_INPUT_FP))]
	pub file_path: String,

	/// Enable debug information
	#[arg(long, short, default_value_t = false)]
	pub debug: bool
}
