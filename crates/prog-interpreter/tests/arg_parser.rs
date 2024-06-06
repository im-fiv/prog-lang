use prog_interpreter::arg_parser::{ArgList, Arg, ParsedArg, ArgumentParseError};
use prog_interpreter::values::*;
use prog_macros::get_argument;

#[test]
fn parse_empty() {
	let arg_list = ArgList::new_empty();
	let parsed = arg_list.verify(&[]);
	
	assert!(parsed.is_ok());
}

#[test]
fn parse_required() {
	let arg_list = ArgList::new(vec![
		Arg::Required("arg1", RuntimeValueKind::Boolean),
		Arg::Required("arg2", RuntimeValueKind::Number),
		Arg::Required("arg3", RuntimeValueKind::String)
	]);

	let parsed = arg_list.verify(&[
		RuntimeValue::Boolean(RuntimeBoolean(true).into()),
		RuntimeValue::Number(RuntimeNumber(3.14).into()),
		RuntimeValue::String(RuntimeString(String::from("hello, world!")).into())
	]);

	assert!(parsed.is_ok());

	let parsed = parsed.unwrap();

	assert!(parsed.contains_key("arg1"));
	assert!(parsed.contains_key("arg2"));
	assert!(parsed.contains_key("arg3"));
}

#[test]
fn parse_mixed() {
	let arg_list = ArgList::new(vec![
		Arg::Required("arg1", RuntimeValueKind::Boolean),
		Arg::Required("arg2", RuntimeValueKind::Number),
		Arg::Optional("arg3", RuntimeValueKind::String)
	]);

	let parsed = arg_list.verify(&[
		RuntimeValue::Boolean(RuntimeBoolean(true).into()),
		RuntimeValue::Number(RuntimeNumber(3.14).into())
	]);

	assert!(parsed.is_ok());

	let parsed = parsed.unwrap();

	assert!(parsed.contains_key("arg1"));
	assert!(parsed.contains_key("arg2"));
	assert!(!parsed.contains_key("arg3"));
}

#[test]
fn parse_variadic() {
	let arg_list = ArgList::new(vec![
		Arg::Required("arg1", RuntimeValueKind::Boolean),
		Arg::Required("arg2", RuntimeValueKind::Number),
		Arg::Variadic("arg3")
	]);

	let parsed = arg_list.verify(&[
		RuntimeValue::Boolean(RuntimeBoolean(true).into()),
		RuntimeValue::Number(RuntimeNumber(3.14).into()),
		// Variadic arguments
		RuntimeValue::Object(RuntimeObject(std::collections::HashMap::from([
			(String::from("hello"), RuntimeValue::Boolean(RuntimeBoolean(true).into())),
			(String::from("world"), RuntimeValue::Number(RuntimeNumber(5.0).into()))
		])).into()),
		RuntimeValue::Function(RuntimeFunction {
			ast: Box::new(prog_parser::ast::expressions::Function {
				arguments: vec![],
				statements: vec![],
				position: 0..0
			}),
			source: String::new(),
			file: String::new()
		}.into())
	]);

	assert!(parsed.is_ok());

	let parsed = parsed.unwrap();
	let arg3 = get_argument!(parsed => arg3: ...);

	assert_eq!(arg3.len(), 2)
}

#[test]
fn getting_arguments() {
	let arg_list = ArgList::new(vec![
		Arg::Required("arg1", RuntimeValueKind::Boolean),
		Arg::Required("arg2", RuntimeValueKind::Number)
	]);

	let parsed = arg_list.verify(&[
		RuntimeValue::Boolean(RuntimeBoolean(true).into()),
		RuntimeValue::Number(RuntimeNumber(3.14).into())
	]);

	assert!(parsed.is_ok());

	let parsed = parsed.unwrap();

	let arg1 = get_argument!(parsed => arg1: RuntimeBoolean);
	let arg2 = get_argument!(parsed => arg2: RuntimeNumber);

	assert_eq!(arg1.owned(), true);
	assert_eq!(arg2.owned(), 3.14);
}

#[test]
#[should_panic]
fn getting_nonexistent_arguments() {
	let arg_list = ArgList::new(vec![
		Arg::Required("arg1", RuntimeValueKind::Boolean)
	]);

	let parsed = arg_list.verify(&[
		RuntimeValue::Boolean(RuntimeBoolean(true).into())
	]);

	assert!(parsed.is_ok());

	let parsed = parsed.unwrap();

	get_argument!(parsed => arg1: RuntimeBoolean);
	get_argument!(parsed => arg2: RuntimeString);
}

#[test]
#[should_panic]
fn getting_arguments_incorrectly() {
	let arg_list = ArgList::new(vec![
		Arg::Required("arg1", RuntimeValueKind::Boolean)
	]);

	let parsed = arg_list.verify(&[
		RuntimeValue::Boolean(RuntimeBoolean(true).into())
	]);

	assert!(parsed.is_ok());

	let parsed = parsed.unwrap();

	get_argument!(parsed => arg1: ...);
}

#[test]
fn parse_count_mismatch() {
	let arg_list = ArgList::new_empty();

	let parsed = arg_list.verify(&[
		RuntimeValue::String(RuntimeString(String::from("this argument shouldn't be here!")).into())
	]);

	assert!(parsed.is_err());

	let error = parsed.unwrap_err();

	if let ArgumentParseError::CountMismatch { expected, end_boundary, got } = error {
		assert_eq!(expected, 0..0);
		assert_eq!(end_boundary, true);
		assert_eq!(got, 1);
	} else {
		panic!("Error is not of type `CountMismatch`");
	}
}

#[test]
fn parse_incorrect_type() {
	let arg_list = ArgList::new(vec![
		Arg::Required("arg1", RuntimeValueKind::Number)
	]);

	let parsed = arg_list.verify(&[
		RuntimeValue::String(RuntimeString(String::from("this argument is of incorrect type!")).into())
	]);

	assert!(parsed.is_err());

	let error = parsed.unwrap_err();

	if let ArgumentParseError::IncorrectType { index, expected, got } = error {
		assert_eq!(index, 0);
		assert_eq!(expected, RuntimeValueKind::Number.to_string());
		assert_eq!(got, RuntimeValueKind::String.to_string());
	} else {
		panic!("Error is not of type `IncorrectType`");
	}
}

#[test]
#[should_panic]
fn invalid_arg_list() {
	let arg_list = ArgList::new(vec![
		Arg::Variadic("variadic1"),
		Arg::Variadic("variadic2")
	]);

	let _ = arg_list.verify(&[
		RuntimeValue::String(RuntimeString(String::from("this test should panic")).into()),
		RuntimeValue::String(RuntimeString(String::from("so it doesn't matter")).into()),
		RuntimeValue::String(RuntimeString(String::from("which arguments we input")).into())
	]);
}