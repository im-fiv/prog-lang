use std::collections::HashMap;

use anyhow::{bail, Result};
use prog_macros::get_argument;

use crate::arg_parser::{Arg, ArgList, ParsedArg};
use crate::errors;
use crate::values::*;

//* Note: this is an extension for convenient value insertion
trait QuickInsert {
	fn quick_insert(
		&mut self,
		key: impl Into<String>,
		value: impl Into<Value>,
		bring_into_scope: bool
	);
}

impl QuickInsert for HashMap<String, (Value, bool)> {
	fn quick_insert(
		&mut self,
		key: impl Into<String>,
		value: impl Into<Value>,
		bring_into_scope: bool
	) {
		self.insert(key.into(), (value.into(), bring_into_scope));
	}
}

fn print_function(
	RIntrinsicFunctionData {
		interpreter,
		arguments,
		..
	}: RIntrinsicFunctionData
) -> Result<Value> {
	let to_print = get_argument!(arguments => varargs: ...)
		.into_iter()
		.map(|arg| format!("{}", arg))
		.collect::<Vec<String>>()
		.join("");

	interpreter
		.context
		.deref_mut()
		.stdout
		.push_str(&format!("{}\n", to_print)[..]);

	if interpreter.context.deref().flags.con_stdout_allowed {
		println!("{to_print}");
	}

	Ok(Value::Empty)
}

// TODO: shift the path during import
fn import_function(
	RIntrinsicFunctionData {
		interpreter,
		arguments,
		call_site,
		..
	}: RIntrinsicFunctionData
) -> Result<Value> {
	use std::mem::swap;

	if !interpreter.context.deref().flags.imports_allowed {
		bail!(errors::InterpretError::new(
			call_site.source,
			call_site.file,
			call_site.func,
			errors::InterpretErrorKind::ContextDisallowed(errors::ContextDisallowed {
				thing: String::from("imports"),
				plural: true
			})
		));
	}

	let path_str = get_argument!(arguments => path: RString).get_owned();

	let mut path = std::path::Path::new(&path_str).to_path_buf();

	// If specified path is a directory, try to get the core file
	if path.is_dir() {
		path.push("mod.prog");
	}

	if path.extension().is_none() {
		path.set_extension("prog");
	}

	if !path.is_file() {
		bail!(errors::InterpretError::new(
			call_site.source,
			call_site.file,
			call_site.args.unwrap().position(),
			errors::InterpretErrorKind::InvalidFile(errors::InvalidFile(path_str))
		));
	}

	if !path.exists() {
		bail!(errors::InterpretError::new(
			call_site.source,
			call_site.file,
			call_site.args.unwrap().position(),
			errors::InterpretErrorKind::InvalidFile(errors::InvalidFile(path_str))
		));
	}

	// TODO: this is also horrible
	let path_str = unsafe {
		let path = path.to_str().unwrap();
		std::mem::transmute::<&'_ str, &'static str>(path)
	};
	let contents_a = prog_utils::read_file(path_str);
	let contents_static =
		unsafe { std::mem::transmute::<&'_ str, &'static str>(contents_a.as_str()) };
	let ts_a = prog_lexer::lex(contents_static, &path_str)?;
	let ts_static = unsafe {
		std::mem::transmute::<prog_lexer::TokenStream<'_>, prog_lexer::TokenStream<'static>>(ts_a)
	};

	let buffer = unsafe {
		std::mem::transmute::<&'_ [prog_lexer::Token<'_>], &'static [prog_lexer::Token<'static>]>(
			ts_static.buffer()
		)
	};

	let ps = prog_parser::ParseStream::new(buffer);
	let ast = ps.parse::<prog_parser::ast::Program>()?;

	// Swapping the active memory to a new interpreter for the time of execution,
	// such that only 1 memory is getting allocations
	let mut new_interpreter = crate::Interpreter::new();
	swap(&mut new_interpreter.memory, &mut interpreter.memory);

	let result = interpreter.interpret(contents_static, path_str, ast, false)?;
	swap(&mut new_interpreter.memory, &mut interpreter.memory);

	Ok(result)
}

fn input_function(
	RIntrinsicFunctionData {
		interpreter,
		arguments,
		call_site,
		..
	}: RIntrinsicFunctionData
) -> Result<Value> {
	use text_io::read;

	if !interpreter.context.deref().flags.inputs_allowed {
		bail!(errors::InterpretError::new(
			call_site.source,
			call_site.file,
			call_site.func,
			errors::InterpretErrorKind::ContextDisallowed(errors::ContextDisallowed {
				thing: String::from("user input"),
				plural: false
			})
		));
	}

	let message = get_argument!(arguments => message: RString?);

	if let Some(message) = message {
		print!("{}", message.get());
	}

	let mut result: String = read!("{}\n");

	// Sanitize the string (just in case)
	result = result.replace('\r', "");
	result = result.replace('\n', "");

	interpreter
		.context
		.deref_mut()
		.stdin
		.push_str(&format!("{result}\n")[..]);

	Ok(Value::String(result.into()))
}

fn raw_print_function(
	RIntrinsicFunctionData {
		interpreter,
		arguments,
		..
	}: RIntrinsicFunctionData
) -> Result<Value> {
	use std::io;
	use std::io::Write;

	let text = get_argument!(arguments => string: RString).get_owned();
	interpreter.context.deref_mut().stdout.push_str(&text);

	if interpreter.context.deref().flags.con_stdout_allowed {
		print!("{text}");
		io::stdout().flush().unwrap();
	}

	Ok(Value::Empty)
}

fn assert_function(
	RIntrinsicFunctionData {
		arguments,
		call_site,
		..
	}: RIntrinsicFunctionData
) -> Result<Value> {
	let value = get_argument!(arguments => value: Value);
	let message = get_argument!(arguments => message: RString?).map(|str| str.get_owned());

	// TODO: fix `source` and `file` fields being empty
	if !value.is_truthy() {
		bail!(crate::InterpretError::new(
			call_site.source,
			call_site.file,
			call_site.args.unwrap().position(),
			errors::InterpretErrorKind::AssertionFailed(errors::AssertionFailed(message))
		));
	}

	Ok(Value::Empty)
}

fn dump_ctx_function(
	RIntrinsicFunctionData { interpreter, .. }: RIntrinsicFunctionData
) -> Result<Value> {
	println!("{:#?}", interpreter.context);
	Ok(Value::Empty)
}

pub fn create_variable_table() -> HashMap<String, (Value, bool)> {
	let mut map = HashMap::new();

	map.quick_insert(
		"print",
		RIntrinsicFunction::new(print_function, ArgList::new(vec![Arg::Variadic("varargs")])),
		true
	);

	map.quick_insert(
		"import",
		RIntrinsicFunction::new(
			import_function,
			ArgList::new(vec![Arg::Required("path", ValueKind::String)])
		),
		true
	);

	map.quick_insert(
		"input",
		RIntrinsicFunction::new(
			input_function,
			ArgList::new(vec![Arg::Optional("message", ValueKind::String)])
		),
		true
	);

	map.quick_insert(
		"raw_print",
		RIntrinsicFunction::new(
			raw_print_function,
			ArgList::new(vec![Arg::Optional("string", ValueKind::String)])
		),
		false
	);

	map.quick_insert(
		"assert",
		RIntrinsicFunction::new(
			assert_function,
			ArgList::new(vec![
				Arg::Required("value", ValueKind::Boolean),
				Arg::Optional("message", ValueKind::String),
			])
		),
		true
	);

	map.quick_insert(
		"dump_ctx",
		RIntrinsicFunction::new(dump_ctx_function, ArgList::new_empty()),
		false
	);

	map
}
