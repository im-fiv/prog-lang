mod boolean;
mod control_flow;
mod function;
mod list;
mod number;
mod object;
mod string;

pub use boolean::Bool;
pub use control_flow::CtrlFlow;
pub use function::Func;
pub use list::List;
pub use number::Num;
pub use object::Obj;
pub use string::Str;

use std::fmt::{self, Display};

pub trait Primitive {
	fn is_truthy(&self) -> bool;
}

#[derive(Debug, Clone, PartialEq, prog_macros::EnumKind)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Value<'ast> {
	Num(Num),
	Bool(Bool),
	Str(Str),
	Func(Func<'ast>),
	List(List<'ast>),
	Obj(Obj<'ast>),

	CtrlFlow(CtrlFlow<'ast>),
	None
}

impl Value<'_> {
	pub fn is_truthy(&self) -> bool {
		match self {
			Self::Num(num) => num as &dyn Primitive,
			Self::Bool(bool) => bool as &dyn Primitive,
			Self::Str(str) => str as &dyn Primitive,
			Self::Func(func) => func as &dyn Primitive,
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
			Self::List(list) => list as &dyn Display,
			Self::Obj(obj) => obj as &dyn Display,

			Self::CtrlFlow(ctrl) => ctrl as &dyn Display,
			Self::None => return write!(f, "")
		}
		.fmt(f)
	}
}
