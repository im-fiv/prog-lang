use std::collections::HashMap;

use anyhow::{bail, Result};
use prog_macros::get_argument;

use crate::arg_parser::{Arg, ArgList, ParsedArg};
use crate::errors;
use crate::values::*;

fn print_function(
	IntrinsicFunctionData {
		interpreter,
		arguments,
		..
	}: IntrinsicFunctionData
) -> Result<RuntimeValue> {
	let to_print = get_argument!(arguments => varargs: ...)
		.into_iter()
		.map(|arg| format!("{}", arg))
		.collect::<Vec<String>>()
		.join("");

	interpreter
		.context
		.stdout
		.push_str(&format!("{}\n", to_print)[..]);

	if interpreter.context.flags.con_stdout_allowed {
		println!("{to_print}");
	}

	Ok(RuntimeValue::Empty)
}

fn import_function(
	IntrinsicFunctionData {
		interpreter,
		arguments,
		call_site,
		..
	}: IntrinsicFunctionData
) -> Result<RuntimeValue> {
	use std::mem::swap;

	if !interpreter.context.flags.imports_allowed {
		bail!(errors::InterpretError::new(
			call_site.source,
			call_site.file,
			call_site.func_pos,
			errors::InterpretErrorKind::ContextDisallowed(errors::ContextDisallowed {
				thing: String::from("imports"),
				plural: true
			})
		));
	}

	let path_str = get_argument!(arguments => path: RuntimeString).owned();

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

	// Swapping the active memory to a new interpreter for the time of execution,
	// such that only 1 memory is getting allocations
	let mut new_interpreter = crate::Interpreter::new(contents, path_str.to_owned());
	swap(&mut new_interpreter.memory, &mut interpreter.memory);

	let result = interpreter.execute(ast, false)?;
	swap(&mut new_interpreter.memory, &mut interpreter.memory);

	Ok(result)
}

fn input_function(
	IntrinsicFunctionData {
		interpreter,
		arguments,
		call_site,
		..
	}: IntrinsicFunctionData
) -> Result<RuntimeValue> {
	use text_io::read;

	if !interpreter.context.flags.inputs_allowed {
		bail!(errors::InterpretError::new(
			call_site.source,
			call_site.file,
			call_site.func_pos,
			errors::InterpretErrorKind::ContextDisallowed(errors::ContextDisallowed {
				thing: String::from("user input"),
				plural: false
			})
		));
	}

	let message = get_argument!(arguments => message: RuntimeString?);

	if let Some(message) = message {
		print!("{}", message.value());
	}

	let mut result: String = read!("{}\n");

	// Sanitize the string (just in case)
	result = result.replace('\r', "");
	result = result.replace('\n', "");

	interpreter
		.context
		.stdin
		.push_str(&format!("{result}\n")[..]);

	Ok(RuntimeValue::String(result.into()))
}

fn raw_print_function(
	IntrinsicFunctionData {
		interpreter,
		arguments,
		..
	}: IntrinsicFunctionData
) -> Result<RuntimeValue> {
	use std::io;
	use std::io::Write;

	let text = get_argument!(arguments => string: RuntimeString).owned();
	interpreter.context.stdout.push_str(&text);

	if interpreter.context.flags.con_stdout_allowed {
		print!("{text}");
		io::stdout().flush().unwrap();
	}

	Ok(RuntimeValue::Empty)
}

fn assert_function(
	IntrinsicFunctionData {
		arguments,
		call_site,
		..
	}: IntrinsicFunctionData
) -> Result<RuntimeValue> {
	let value = get_argument!(arguments => value: RuntimeValue);
	let message = get_argument!(arguments => message: RuntimeString?).map(|str| str.owned());

	if !value.is_truthy() {
		bail!(crate::InterpretError::new(
			call_site.source,
			call_site.file,
			call_site.args_pos,
			errors::InterpretErrorKind::AssertionFailed(errors::AssertionFailed(message))
		));
	}

	Ok(RuntimeValue::Empty)
}

fn dump_ctx_function(
	IntrinsicFunctionData { interpreter, .. }: IntrinsicFunctionData
) -> Result<RuntimeValue> {
	println!("{:#?}", interpreter.context);
	Ok(RuntimeValue::Empty)
}

pub fn create_variable_table() -> HashMap<String, RuntimeValue> {
	let mut map = HashMap::new();

	map.insert(
		String::from("print"),
		IntrinsicFunction::new(print_function, ArgList::new(vec![Arg::Variadic("varargs")])).into()
	);

	map.insert(
		String::from("import"),
		IntrinsicFunction::new(
			import_function,
			ArgList::new(vec![Arg::Required("path", RuntimeValueKind::String)])
		)
		.into()
	);

	map.insert(
		String::from("input"),
		IntrinsicFunction::new(
			input_function,
			ArgList::new(vec![Arg::Optional("message", RuntimeValueKind::String)])
		)
		.into()
	);

	map.insert(
		String::from("raw_print"),
		IntrinsicFunction::new(
			raw_print_function,
			ArgList::new(vec![Arg::Optional("string", RuntimeValueKind::String)])
		)
		.into()
	);

	map.insert(
		String::from("assert"),
		IntrinsicFunction::new(
			assert_function,
			ArgList::new(vec![
				Arg::Required("value", RuntimeValueKind::Boolean),
				Arg::Optional("message", RuntimeValueKind::String),
			])
		)
		.into()
	);

	map.insert(
		String::from("dump_ctx"),
		IntrinsicFunction::new(dump_ctx_function, ArgList::new_empty()).into()
	);

	map
}
