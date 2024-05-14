use std::collections::HashMap;
use std::cell::RefCell;
use anyhow::{Result, bail};

use prog_macros::get_argument;

use crate::values::*;
use crate::arg_parser::{ArgList, Arg, ParsedArg};
use crate::context::RuntimeContext;
use crate::errors;

fn print_function(
	_this: Option<Box<RuntimeValue>>,
	context: &mut RuntimeContext,
	args: HashMap<String, ParsedArg>,
	_call_site: CallSite
) -> Result<RuntimeValue> {
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

fn import_function(
	_this: Option<Box<RuntimeValue>>,
	context: &mut RuntimeContext,
	args: HashMap<String, ParsedArg>,
	call_site: CallSite
) -> Result<RuntimeValue> {
	if !context.imports_allowed {
		bail!(errors::InterpretError::new(
			call_site.source,
			call_site.file,
			call_site.position,
			errors::InterpretErrorKind::ContextDisallowed(errors::ContextDisallowed {
				thing: String::from("imports"),
				plural: true
			})
		));
	}

	let path_str = get_argument!(args => path: RefCell<RuntimeString>)
		.borrow()
		.owned();

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

	let path_str = path.to_str().unwrap();
	let contents = prog_utils::read_file(path_str);

	let parser = prog_parser::Parser::new(&contents, path_str);
	let ast = parser.parse()?;

	let mut interpreter = crate::Interpreter::new(contents, path_str.to_owned());
	context.clone_into(&mut interpreter.context);

	let result = interpreter.execute(ast, false)?;

	*context = interpreter.context;

	Ok(result)
}

fn input_function(
	_this: Option<Box<RuntimeValue>>,
	context: &mut RuntimeContext,
	args: HashMap<String, ParsedArg>,
	call_site: CallSite
) -> Result<RuntimeValue> {
	use text_io::read;

	if !context.input_allowed {
		bail!(errors::InterpretError::new(
			call_site.source,
			call_site.file,
			call_site.position,
			errors::InterpretErrorKind::ContextDisallowed(errors::ContextDisallowed {
				thing: String::from("user input"),
				plural: false
			})
		));
	}

	let message = get_argument!(args => message: RefCell<RuntimeString>?)
		.and_then(|arg| Some(
			arg.borrow().owned()
		));

	if let Some(message) = message {
		print!("{}", &message[..]);
	}

	let mut result: String = read!("{}\n");

	// Sanitize the string (just in case)
	result = result.replace('\r', "");
	result = result.replace('\n', "");

	context.stdin.push_str(&format!("{result}\n")[..]);

	Ok(RuntimeValue::String(
		RuntimeString::from(result).into()
	))
}

fn raw_print_function(
	_this: Option<Box<RuntimeValue>>,
	_context: &mut RuntimeContext,
	args: HashMap<String, ParsedArg>,
	_call_site: CallSite
) -> Result<RuntimeValue> {
	use std::io;
	use std::io::Write;
	
	let to_print = get_argument!(args => string: RefCell<RuntimeString>)
		.borrow()
		.owned();

	print!("{to_print}");
	io::stdout().flush().unwrap();

	Ok(RuntimeValue::Empty)
}

pub fn create_value_table() -> HashMap<String, RuntimeValue> {
	let mut map = HashMap::new();

	map.insert(String::from("print"), IntrinsicFunction::new(
		print_function,
		ArgList::new(vec![
			Arg::Variadic("varargs")
		])
	).into());

	map.insert(String::from("import"), IntrinsicFunction::new(
		import_function,
		ArgList::new(vec![
			Arg::Required("path", RuntimeValueKind::String)
		])
	).into());

	map.insert(String::from("input"), IntrinsicFunction::new(
		input_function,
		ArgList::new(vec![
			Arg::Optional("message", RuntimeValueKind::String)
		])
	).into());

	map.insert(String::from("raw_print"), IntrinsicFunction::new(
		raw_print_function,
		ArgList::new(vec![
			Arg::Optional("string", RuntimeValueKind::String)
		])
	).into());

	map
}