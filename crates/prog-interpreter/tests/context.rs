use prog_interpreter::context::Context;
use prog_interpreter::values::*;

#[test]
fn insert_and_get_variable() {
	let some_value = Value::Boolean(true.into());

	let mut context = Context::new();
	context.insert(String::from("some_variable"), some_value.clone());

	let got_value = context.get("some_variable").unwrap();

	assert_eq!(got_value, some_value)
}

#[test]
fn update_variable() {
	let some_value = Value::Boolean(true.into());

	let mut context = Context::new();
	context.insert(String::from("some_variable"), some_value.clone());

	let new_value = Value::Boolean(false.into());
	let old_value = context
		.update(String::from("some_variable"), new_value.clone())
		.unwrap();

	let got_value = context.get("some_variable").unwrap();

	assert_eq!(old_value, some_value);
	assert_eq!(got_value, new_value);
}

#[test]
#[should_panic]
fn get_nonexistent_variable() {
	let context = Context::new();
	let _ = context.get(&"some_variable").unwrap();
}

#[test]
fn get_variable_mutable() {
	let some_value = Value::Boolean(true.into());

	let mut context = Context::new();
	context.insert(String::from("some_variable"), some_value.clone());

	let value_ref = context.get_mut("some_variable").unwrap();

	if let Value::Boolean(inner_value) = value_ref {
		// Inverting the value
		*inner_value = (!inner_value.get_owned()).into();
	} else {
		panic!("Value is not of type Boolean");
	}

	let got_value = context.get("some_variable").unwrap();
	assert_eq!(got_value, Value::Boolean(false.into()));
}

#[test]
fn subcontexts() {
	let some_value = Value::Boolean(true.into());

	let mut context = Context::new();

	context.deeper();
	context.insert(String::from("some_variable"), some_value);
	context.shallower();

	let result = context.get("some_value");
	assert!(result.is_err());
}
