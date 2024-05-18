use prog_interpreter::context::RuntimeContext;
use prog_interpreter::values::*;

#[test]
fn insert_and_get_value() {
	let some_value = RuntimeValue::Boolean(RuntimeBoolean(true).into());

	let mut context = RuntimeContext::new_clean();
	context.insert_value(String::from("some_value"), some_value.clone()).unwrap();

	let got_value = context.get_value(&String::from("some_value")).unwrap();

	assert_eq!(got_value, some_value)
}

#[test]
fn update_value() {
	let some_value = RuntimeValue::Boolean(RuntimeBoolean(true).into());

	let mut context = RuntimeContext::new_clean();
	context.insert_value(String::from("some_value"), some_value.clone()).unwrap();

	let new_value = RuntimeValue::Boolean(RuntimeBoolean(false).into());
	let old_value = context.update_value(String::from("some_value"), new_value.clone()).unwrap();

	let got_value = context.get_value(&String::from("some_value")).unwrap();

	assert_eq!(old_value, some_value);
	assert_eq!(got_value, new_value);
}

#[test]
#[should_panic]
fn get_nonexistent_value() {
	let context = RuntimeContext::new_clean();
	let _ = context.get_value(&String::from("some_value")).unwrap();
}

#[test]
#[should_panic]
fn insert_twice() {
	let some_value = RuntimeValue::Boolean(RuntimeBoolean(true).into());

	// Due to the way current logic is implemented, the value must exist both
	// in the global table, and some subcontext for it to panic during insertion.
	// I don't want to change it due to how nicely shadowing works with the current logic
	let mut context = RuntimeContext::new_clean();
	context.insert_value(String::from("some_value"), some_value.clone()).unwrap();
	context.deeper();
	context.insert_value(String::from("some_value"), some_value.clone()).unwrap();

	context.insert_value(String::from("some_value"), some_value).unwrap();
}

#[test]
fn get_value_mutable() {
	let some_value = RuntimeValue::Boolean(RuntimeBoolean(true).into());

	let mut context = RuntimeContext::new_clean();
	context.insert_value(String::from("some_value"), some_value.clone()).unwrap();

	let value_ref = context.get_value_mut(&String::from("some_value")).unwrap();

	if let RuntimeValue::Boolean(inner_value) = value_ref {
		let mut borrowed_inner_value = inner_value.borrow_mut();

		// Inverting the value
		*borrowed_inner_value = RuntimeBoolean(!borrowed_inner_value.0);
	} else {
		panic!("Value is not of type Boolean");
	}

	let got_value = context.get_value(&String::from("some_value")).unwrap();
	assert_eq!(got_value, RuntimeValue::Boolean(RuntimeBoolean(false).into()));
}

#[test]
fn subcontexts() {
	let some_value = RuntimeValue::Boolean(RuntimeBoolean(true).into());

	let mut context = RuntimeContext::new_clean();

	context.deeper();
	context.insert_value(String::from("some_value"), some_value).unwrap();
	context.shallower();

	let result = context.get_value(&String::from("some_value"));
	assert!(result.is_err());
}