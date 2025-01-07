use prog_macros::get_argument;
use prog_old_interpreter::arg_parser::{Arg, ArgList, ArgumentParseError, ParsedArg};
use prog_old_interpreter::values::*;

#[test]
fn parse_empty() {
	let arg_list = ArgList::new_empty();
	let parsed = arg_list.verify(&[]);

	assert!(parsed.is_ok());
}

#[test]
fn parse_required() {
	let arg_list = ArgList::new(vec![
		Arg::Required("arg1", ValueKind::Boolean),
		Arg::Required("arg2", ValueKind::Number),
		Arg::Required("arg3", ValueKind::String),
	]);

	let parsed = arg_list.verify(&[
		Value::Boolean(true.into()),
		Value::Number(3.14.into()),
		Value::String(String::from("hello, world!").into())
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
		Arg::Required("arg1", ValueKind::Boolean),
		Arg::Required("arg2", ValueKind::Number),
		Arg::Optional("arg3", ValueKind::String),
	]);

	let parsed = arg_list.verify(&[Value::Boolean(true.into()), Value::Number(3.14.into())]);

	assert!(parsed.is_ok());

	let parsed = parsed.unwrap();

	assert!(parsed.contains_key("arg1"));
	assert!(parsed.contains_key("arg2"));
	assert!(!parsed.contains_key("arg3"));
}

#[test]
fn parse_variadic() {
	let arg_list = ArgList::new(vec![
		Arg::Required("arg1", ValueKind::Boolean),
		Arg::Required("arg2", ValueKind::Number),
		Arg::Variadic("arg3"),
	]);

	let parsed = arg_list.verify(&[
		Value::Boolean(true.into()),
		Value::Number(3.14.into()),
		// Variadic arguments
		Value::Boolean(true.into()),
		Value::Number(3.14.into()),
		Value::Boolean(true.into()),
		Value::Number(3.14.into())
	]);

	assert!(parsed.is_ok());

	let parsed = parsed.unwrap();
	let arg3 = get_argument!(parsed => arg3: ...);

	assert_eq!(arg3.len(), 4)
}

#[test]
fn getting_arguments() {
	let arg_list = ArgList::new(vec![
		Arg::Required("arg1", ValueKind::Boolean),
		Arg::Required("arg2", ValueKind::Number),
	]);

	let parsed = arg_list.verify(&[Value::Boolean(true.into()), Value::Number(3.14.into())]);

	assert!(parsed.is_ok());

	let parsed = parsed.unwrap();

	let arg1 = get_argument!(parsed => arg1: RBoolean);
	let arg2 = get_argument!(parsed => arg2: RNumber);

	assert_eq!(arg1.get_owned(), true);
	assert_eq!(arg2.get_owned(), 3.14);
}

#[test]
#[should_panic]
fn getting_nonexistent_arguments() {
	let arg_list = ArgList::new(vec![Arg::Required("arg1", ValueKind::Boolean)]);

	let parsed = arg_list.verify(&[Value::Boolean(true.into())]);

	assert!(parsed.is_ok());

	let parsed = parsed.unwrap();

	get_argument!(parsed => arg1: RBoolean);
	get_argument!(parsed => arg2: RString);
}

#[test]
#[should_panic]
fn getting_arguments_incorrectly() {
	let arg_list = ArgList::new(vec![Arg::Required("arg1", ValueKind::Boolean)]);

	let parsed = arg_list.verify(&[Value::Boolean(true.into())]);

	assert!(parsed.is_ok());

	let parsed = parsed.unwrap();

	get_argument!(parsed => arg1: ...);
}

#[test]
fn parse_count_mismatch() {
	let arg_list = ArgList::new_empty();

	let parsed = arg_list.verify(&[Value::String(
		String::from("this argument shouldn't be here!").into()
	)]);

	assert!(parsed.is_err());

	let error = parsed.unwrap_err();

	if let ArgumentParseError::CountMismatch {
		expected,
		end_boundary,
		got
	} = error
	{
		assert_eq!(expected, 0..0);
		assert_eq!(end_boundary, true);
		assert_eq!(got, 1);
	} else {
		panic!("Error is not of type `CountMismatch`");
	}
}

#[test]
fn parse_incorrect_type() {
	let arg_list = ArgList::new(vec![Arg::Required("arg1", ValueKind::Number)]);

	let parsed = arg_list.verify(&[Value::String(
		String::from("this argument is of incorrect type!").into()
	)]);

	assert!(parsed.is_err());

	let error = parsed.unwrap_err();

	if let ArgumentParseError::IncorrectType {
		index,
		expected,
		got
	} = error
	{
		assert_eq!(index, 0);
		assert_eq!(expected, ValueKind::Number.to_string());
		assert_eq!(got, ValueKind::String.to_string());
	} else {
		panic!("Error is not of type `IncorrectType`");
	}
}

#[test]
#[should_panic]
fn invalid_arg_list() {
	let arg_list = ArgList::new(vec![Arg::Variadic("variadic1"), Arg::Variadic("variadic2")]);

	let _ = arg_list.verify(&[
		Value::String(String::from("this test should panic").into()),
		Value::String(String::from("so it doesn't matter").into()),
		Value::String(String::from("which arguments we input").into())
	]);
}
