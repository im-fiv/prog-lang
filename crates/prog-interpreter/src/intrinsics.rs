use prog_macros::get_argument;
use prog_parser::ASTNode;

use crate::arg_parser::{Arg, ArgList};
use crate::value::{CallableData, IntrinsicFn};
use crate::{error, InterpretError, InterpretResult, Primitive, Value, ValueKind};

#[derive(Debug)]
pub(crate) struct Intrinsic<'i> {
	pub(crate) name: &'static str,
	pub(crate) value: Value<'i>,
	pub(crate) auto_import: bool
}

#[derive(Debug)]
pub(crate) struct IntrinsicTable<'i> {
	entries: Vec<Intrinsic<'i>>
}

impl<'i> IntrinsicTable<'i> {
	pub fn new() -> Self {
		let mut this = Self::new_empty();
		this.entries.extend(Self::fetch());

		this
	}

	pub fn new_empty() -> Self { Self { entries: vec![] } }

	fn fetch() -> Box<[Intrinsic<'i>]> {
		Box::new([
			#[cfg(debug_assertions)]
			Intrinsic {
				name: "should_panic",
				value: Value::IntrinsicFn(IntrinsicFn::new(
					i_should_panic,
					ArgList::new(vec![Arg::Required("func".into(), ValueKind::Func)])
				)),
				auto_import: false
			},
			Intrinsic {
				name: "print",
				value: Value::IntrinsicFn(IntrinsicFn::new(
					i_print,
					ArgList::new(vec![Arg::Variadic("args".into())])
				)),
				auto_import: true
			},
			Intrinsic {
				name: "debug",
				value: Value::IntrinsicFn(IntrinsicFn::new(
					i_debug,
					ArgList::new(vec![Arg::RequiredUntyped("value".into())])
				)),
				auto_import: true
			},
			Intrinsic {
				name: "assert",
				value: Value::IntrinsicFn(IntrinsicFn::new(
					i_assert,
					ArgList::new(vec![
						Arg::Required("expr".into(), ValueKind::Bool),
						Arg::Optional("msg".into(), ValueKind::Str),
					])
				)),
				auto_import: true
			},
			Intrinsic {
				name: "assert_eq",
				value: Value::IntrinsicFn(IntrinsicFn::new(
					i_assert_eq,
					ArgList::new(vec![
						Arg::RequiredUntyped("left".into()),
						Arg::RequiredUntyped("right".into()),
					])
				)),
				auto_import: true
			},
			Intrinsic {
				name: "assert_neq",
				value: Value::IntrinsicFn(IntrinsicFn::new(
					i_assert_neq,
					ArgList::new(vec![
						Arg::RequiredUntyped("left".into()),
						Arg::RequiredUntyped("right".into()),
					])
				)),
				auto_import: true
			}
		])
	}
}

impl<'i> IntoIterator for IntrinsicTable<'i> {
	type IntoIter = std::vec::IntoIter<Self::Item>;
	type Item = Intrinsic<'i>;

	fn into_iter(self) -> Self::IntoIter { self.entries.into_iter() }
}

impl Default for IntrinsicTable<'_> {
	fn default() -> Self { Self::new_empty() }
}

#[cfg(debug_assertions)]
fn i_should_panic<'i>(
	CallableData {
		i,
		mut args,
		call_site
	}: CallableData<'_, 'i>
) -> InterpretResult<'i, Value<'i>> {
	use crate::Callable;

	let span_callee = call_site.callee;

	let func = Box::new(get_argument!(args => func: Func));
	let result = func.call(CallableData {
		i,
		args: std::collections::HashMap::new(),
		call_site
	});

	if result.is_ok() {
		return Err(InterpretError::new(
			span_callee,
			crate::InterpretErrorKind::AssertionFailed(error::AssertionFailed(Some(String::from(
				"function did not panic"
			))))
		));
	}

	Ok(Value::None)
}

fn i_print<'i>(
	CallableData { i, mut args, .. }: CallableData<'_, 'i>
) -> InterpretResult<'i, Value<'i>> {
	let formatted = get_argument!(args => args: ...)
		.into_iter()
		.map(|v| format!("{v}"))
		.collect::<Vec<_>>()
		.join(" ");

	i.stdout.extend(formatted.bytes());

	if i.context.inner().flags.con_stdout_allowed {
		println!("{formatted}\n");
	}

	Ok(Value::None)
}

fn i_debug<'i>(
	CallableData {
		i,
		mut args,
		call_site
	}: CallableData<'_, 'i>
) -> InterpretResult<'i, Value<'i>> {
	if !i.context.inner().flags.con_stdout_allowed {
		return Ok(Value::None);
	}

	let expr = call_site.args.nth_item(0).copied().unwrap().value();

	let value = get_argument!(args => value: _);

	let file = call_site.file();
	let position = call_site.args.start();

	let mut column = 1;
	let mut row = 1;

	for (idx, char) in call_site.source().char_indices() {
		if idx >= position {
			break;
		}

		if char == '\n' {
			column += 1;
			row = 1;
		}

		row += 1;
	}

	println!("[{file}:{column}:{row}] {expr} = {value}");

	Ok(Value::None)
}

fn i_assert<'i>(
	CallableData {
		mut args,
		call_site,
		..
	}: CallableData<'_, 'i>
) -> InterpretResult<'i, Value<'i>> {
	let expr = get_argument!(args => expr: Bool);
	let msg = get_argument!(args => msg: Str?);

	if !expr.is_truthy() {
		let expr_span = *call_site.args.nth_item(0).unwrap();

		return Err(InterpretError::new(
			expr_span,
			crate::InterpretErrorKind::AssertionFailed(error::AssertionFailed(msg.map(Into::into)))
		));
	}

	Ok(Value::None)
}

fn i_assert_eq<'i>(data: CallableData<'_, 'i>) -> InterpretResult<'i, Value<'i>> {
	generic_lr_assert(|l, r| l == r, data)
}

fn i_assert_neq<'i>(data: CallableData<'_, 'i>) -> InterpretResult<'i, Value<'i>> {
	generic_lr_assert(|l, r| l != r, data)
}

fn generic_lr_assert<'int, F>(
	pred: F,
	CallableData {
		mut args,
		call_site,
		..
	}: CallableData<'_, 'int>
) -> InterpretResult<'int, Value<'int>>
where
	F: FnOnce(&Value<'int>, &Value<'int>) -> bool
{
	use prog_parser::{Position, Span};

	let left = get_argument!(args => left: _);
	let right = get_argument!(args => right: _);

	if !pred(&left, &right) {
		let source = call_site.args.source();
		let file = call_site.args.file();

		let span_left = *call_site.args.items().first().copied().unwrap();
		let span_right = *call_site.args.items().get(1).copied().unwrap();

		let start = span_left.position().start();
		let end = span_right.position().end();

		let position = Position::new(start, end);
		let expr_span = Span::new(source, file, position);

		return Err(InterpretError::new(
			expr_span,
			crate::InterpretErrorKind::AssertionEqFailed(error::AssertionEqFailed {
				left: (left, span_left),
				right: (right, span_right)
			})
		));
	}

	Ok(Value::None)
}
