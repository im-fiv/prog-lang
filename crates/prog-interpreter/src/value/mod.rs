pub(crate) mod boolean;
pub(crate) mod control_flow;
pub(crate) mod function;
pub(crate) mod intrinsic_func;
pub(crate) mod list;
pub(crate) mod number;
pub(crate) mod object;
pub(crate) mod string;

pub use boolean::Bool;
pub use control_flow::CtrlFlow;
pub use function::Func;
pub use intrinsic_func::IntrinsicFn;
pub use list::List;
pub use number::Num;
pub use object::Obj;
pub use string::Str;

use std::fmt::{self, Display};

use prog_parser::{ast, Span};

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

pub trait Primitive {
	fn is_truthy(&self) -> bool;
}

pub(crate) trait Callable<'intref, 'int: 'intref>: Primitive {
	fn arg_list(&self) -> &crate::arg_parser::ArgList;

	fn call(
		self: Box<Self>,
		data: CallableData<'intref, 'int>
	) -> crate::InterpretResult<'int, Value<'int>>;
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

			Self::CtrlFlow(_) => return true,
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
