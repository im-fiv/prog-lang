mod boolean;
mod control_flow;
mod function;
mod number;
mod string;

pub use boolean::Bool;
pub use control_flow::CtrlFlow;
pub use function::Func;
pub use number::Num;
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

	CtrlFlow(CtrlFlow<'ast>),
	None
}

impl Value<'_> {
	pub fn is_truthy(&self) -> bool {
		match self {
			Self::Num(v) => v as &dyn Primitive,
			Self::Bool(v) => v as &dyn Primitive,
			Self::Str(v) => v as &dyn Primitive,
			Self::Func(v) => v as &dyn Primitive,

			Self::CtrlFlow(_) => return true,
			Self::None => return false
		}
		.is_truthy()
	}
}

impl Display for Value<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Num(v) => v as &dyn Display,
			Self::Bool(v) => v as &dyn Display,
			Self::Str(v) => v as &dyn Display,
			Self::Func(v) => v as &dyn Display,

			Self::CtrlFlow(v) => v as &dyn Display,
			Self::None => return write!(f, "")
		}
		.fmt(f)
	}
}
