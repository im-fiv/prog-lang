use prog_macros::get_argument;

use crate::{error, InterpretResult, Value, ValueKind, Primitive, InterpretError};
use crate::value::{IntrinsicFn, CallableData};
use crate::arg_parser::{ArgList, Arg};

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

	pub fn new_empty() -> Self {
		Self { entries: vec![] }
	}

	fn fetch() -> Box<[Intrinsic<'i>]> {
		Box::new([
			Intrinsic {
				name: "print",
				value: Value::IntrinsicFn(IntrinsicFn::new(
					i_print,
					ArgList::new(vec![
						Arg::Variadic("args".into())
					])
				)),
				auto_import: true
			},

			Intrinsic {
				name: "assert",
				value: Value::IntrinsicFn(IntrinsicFn::new(
					i_assert,
					ArgList::new(vec![
						Arg::Required("expr".into(), ValueKind::Bool),
						Arg::Optional("msg".into(), ValueKind::Str)
					])
				)),
				auto_import: true
			}
		])
	}
}

impl<'i> IntoIterator for IntrinsicTable<'i> {
	type Item = Intrinsic<'i>;
	type IntoIter = std::vec::IntoIter<Self::Item>;

	fn into_iter(self) -> Self::IntoIter { self.entries.into_iter() }
}

impl Default for IntrinsicTable<'_> {
	fn default() -> Self { Self::new_empty() }
}

fn i_print<'i>(
	CallableData {
		i,
		mut args,
		..
	}: CallableData<'_, 'i>
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
			crate::InterpretErrorKind::AssertionFailed(error::AssertionFailed(
				msg.map(Into::into)
			))
		));
	}

	Ok(Value::None)
}