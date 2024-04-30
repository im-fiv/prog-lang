use std::collections::HashMap;
use anyhow::{Result, bail};

use prog_macros::get_argument;
use crate::arg_parser::{ArgList, Arg, ParsedArg};
use crate::context::RuntimeContext;
use crate::values::{IntrinsicFunction, RuntimeValue, RuntimeValueKind};

fn print_function(context: &mut RuntimeContext, args: HashMap<String, ParsedArg>) -> Result<RuntimeValue> {
	let to_print = get_argument!(args => varargs: ...)
		.into_iter()
		.map(|arg| format!("{}", arg))
		.collect::<Vec<String>>()
		.join("");

	context.stdout.push_str(&format!("{}\n", to_print)[..]);

	if context.con_stdout_allowed {
		println!("{to_print}");
	}

	Ok(RuntimeValue::Empty)
}

fn import_function(context: &mut RuntimeContext, args: HashMap<String, ParsedArg>) -> Result<RuntimeValue> {
	if !context.imports_allowed {
		bail!("Imports in this context are not allowed");
	}

	let path_str = get_argument!(args => path: String);
	let mut path = std::path::Path::new(&path_str).to_path_buf();

	// If specified path is a directory, try to get the core file
	if path.is_dir() {
		path.push("mod.prog");
	}

	if path.extension().is_none() {
		path.set_extension("prog");
	}

	if !path.is_file() {
		bail!("'{path_str}' is not a valid file");
	}

	if !path.exists() {
		bail!("Cannot find the specified file at path '{path_str}'");
	}

	let contents = prog_utils::read_file(path.to_str().unwrap());
	let ast = prog_parser::parse(&contents)?;
	let mut interpreter = crate::Interpreter::new();
	context.clone_into(&mut interpreter.context);
	let result = interpreter.execute(ast)?;
	*context = interpreter.context;

	Ok(result)
}

fn input_function(context: &mut RuntimeContext, args: HashMap<String, ParsedArg>) -> Result<RuntimeValue> {
	use text_io::read;

	if !context.input_allowed {
		bail!("Input in this context is not allowed");
	}

	let message = get_argument!(args => message: String?);
	if let Some(message) = message {
		print!("{}", &message[..]);
	}

	let mut result: String = read!("{}\n");

	// Sanitize the string (just in case)
	result = result.replace('\r', "");
	result = result.replace('\n', "");

	context.stdin.push_str(&format!("{result}\n")[..]);

	Ok(RuntimeValue::String(result))
}

pub fn create_value_table() -> HashMap<String, RuntimeValue> {
	let mut map = HashMap::new();

	map.insert(
		String::from("print"),
		RuntimeValue::IntrinsicFunction(IntrinsicFunction {
			pointer: print_function,
			arguments: ArgList::new(vec![
				Arg::Variadic("varargs")
			])
		})
	);

	map.insert(
		String::from("import"),
		RuntimeValue::IntrinsicFunction(IntrinsicFunction {
			pointer: import_function,
			arguments: ArgList::new(vec![
				Arg::Required("path", RuntimeValueKind::String)
			])
		})
	);

	map.insert(
		String::from("input"),
		RuntimeValue::IntrinsicFunction(IntrinsicFunction {
			pointer: input_function,
			arguments: ArgList::new(vec![
				Arg::Optional("message", RuntimeValueKind::String)
			])
		})
	);

	map
}