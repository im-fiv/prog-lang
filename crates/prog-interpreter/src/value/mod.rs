pub(crate) mod boolean;
pub(crate) mod class;
pub(crate) mod control_flow;
pub(crate) mod function;
pub(crate) mod intrinsic_func;
pub(crate) mod list;
pub(crate) mod number;
pub(crate) mod object;
pub(crate) mod string;

pub use boolean::Bool;
pub use class::{Class, ClassInstance};
pub use control_flow::CtrlFlow;
pub use function::Func;
pub use intrinsic_func::IntrinsicFn;
pub use list::List;
pub use number::Num;
pub use object::Obj;
pub use string::Str;

use std::borrow::Cow;
use std::fmt::{self, Display};

use prog_parser::{ast, Span};

pub trait AsRaw {
	type Inner;

	fn as_raw(&self) -> &Self::Inner;
}

/// Represents valid runtime values.
pub trait Primitive {
	/// When necessary, the value will be coerced to a boolean based on the return value of this function.
	fn is_truthy(&self) -> bool;
}

/// Represents runtime values which can be invoked.
pub(crate) trait Callable<'intref, 'int: 'intref>: Primitive {
	fn arg_list(&self) -> Cow<crate::arg_parser::ArgList>;

	fn call(
		&mut self,
		data: CallableData<'intref, 'int>
	) -> crate::InterpretResult<'int, Value<'int>>;
}

#[derive(Debug)]
pub(crate) struct CallableData<'intref, 'int: 'intref> {
	pub(crate) i: &'intref mut crate::Interpreter<'int>,
	pub(crate) args: crate::arg_parser::ParsedArgList<'int>,
	pub(crate) call_site: CallSite<'int>
}

#[derive(Debug, Clone)]
pub(crate) struct CallSite<'s> {
	pub(crate) callee: Span<'s>,
	pub(crate) _lp: Span<'s>,
	pub(crate) args: ast::Punctuated<'s, Span<'s>, Span<'s>>,
	pub(crate) _rp: Span<'s>
}

impl<'s> prog_parser::ASTNode<'s> for CallSite<'s> {
	fn span<'a>(&'a self) -> Span<'s> {
		let source = self.callee.source();
		let file = self.callee.file();

		let start = self.callee.position().start();
		let end = self._rp.position().end();

		Span::new(source, file, prog_parser::Position::new(start, end))
	}
}

#[derive(Debug, Clone, PartialEq, prog_macros::EnumKind)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Value<'i> {
	Num(Num),
	Bool(Bool),
	Str(Str),
	Func(Func<'i>),
	IntrinsicFn(IntrinsicFn<'i>),
	List(List<'i>),
	Obj(Obj<'i>),
	Class(Class<'i>),
	ClassInstance(ClassInstance<'i>),

	CtrlFlow(CtrlFlow<'i>),
	None
}

impl Value<'_> {
	pub fn is_truthy(&self) -> bool {
		match self {
			Self::Num(num) => num as &dyn Primitive,
			Self::Bool(bool) => bool as &dyn Primitive,
			Self::Str(str) => str as &dyn Primitive,
			Self::Func(func) => func as &dyn Primitive,
			Self::IntrinsicFn(_) => return true,
			Self::List(list) => list as &dyn Primitive,
			Self::Obj(obj) => obj as &dyn Primitive,
			Self::Class(class) => class as &dyn Primitive,
			Self::ClassInstance(class_inst) => class_inst as &dyn Primitive,

			Self::CtrlFlow(_) => return false,
			Self::None => return false
		}
		.is_truthy()
	}
}

impl Display for Value<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Num(num) => num as &dyn Display,
			Self::Bool(bool) => bool as &dyn Display,
			Self::Str(str) => str as &dyn Display,
			Self::Func(func) => func as &dyn Display,
			Self::IntrinsicFn(func) => func as &dyn Display,
			Self::List(list) => list as &dyn Display,
			Self::Obj(obj) => obj as &dyn Display,
			Self::Class(class) => class as &dyn Display,
			Self::ClassInstance(class_inst) => class_inst as &dyn Display,

			Self::CtrlFlow(ctrl) => ctrl as &dyn Display,
			Self::None => {
				if f.alternate() {
					return write!(f, "{}", prog_lexer::TokenKind::None);
				} else {
					return write!(f, "");
				}
			}
		}
		.fmt(f)
	}
}

impl Default for Value<'_> {
	fn default() -> Self { Self::None }
}

impl From<Num> for Value<'_> {
	fn from(num: Num) -> Self { Self::Num(num) }
}

impl From<Bool> for Value<'_> {
	fn from(bool: Bool) -> Self { Self::Bool(bool) }
}

impl From<Str> for Value<'_> {
	fn from(str: Str) -> Self { Self::Str(str) }
}

impl<'i> From<Func<'i>> for Value<'i> {
	fn from(func: Func<'i>) -> Self { Self::Func(func) }
}

impl<'i> From<IntrinsicFn<'i>> for Value<'i> {
	fn from(func: IntrinsicFn<'i>) -> Self { Self::IntrinsicFn(func) }
}

impl<'i> From<List<'i>> for Value<'i> {
	fn from(list: List<'i>) -> Self { Self::List(list) }
}

impl<'i> From<Obj<'i>> for Value<'i> {
	fn from(obj: Obj<'i>) -> Self { Self::Obj(obj) }
}

impl<'i> From<Class<'i>> for Value<'i> {
	fn from(class: Class<'i>) -> Self { Self::Class(class) }
}

impl<'i> From<ClassInstance<'i>> for Value<'i> {
	fn from(class_inst: ClassInstance<'i>) -> Self { Self::ClassInstance(class_inst) }
}

impl<'i> From<CtrlFlow<'i>> for Value<'i> {
	fn from(ctrl: CtrlFlow<'i>) -> Self { Self::CtrlFlow(ctrl) }
}

impl From<()> for Value<'_> {
	fn from(_: ()) -> Self { Self::None }
}

impl<'i, T> From<Option<T>> for Value<'i>
where
	Value<'i>: From<T>
{
	fn from(opt: Option<T>) -> Self { opt.map_or(Self::None, Self::from) }
}
