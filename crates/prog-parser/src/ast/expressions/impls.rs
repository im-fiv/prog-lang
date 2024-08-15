use std::fmt::{self, Display};
use std::hash::{self, Hash};

use prog_utils::impl_basic_conv;

use super::*;

//* Others *//

impl Expression {
	pub fn position(&self) -> Position {
		match self {
			Self::Unary(value) => value.position.clone(),
			Self::Binary(value) => value.position.clone(),
			Self::Term(value) => value.position(),
			Self::Empty(value) => {
				value
					.to_owned()
					.expect("Position of `Expression::Empty` is `None`")
			}
		}
	}
}

impl Term {
	pub fn position(&self) -> Position {
		match self {
			Self::Object(value) => value.1.clone(),
			Self::List(value) => value.1.clone(),
			Self::Call(value) => value.position.clone(),
			Self::Function(value) => value.position.clone(),
			Self::Literal(value) => value.position(),
			Self::Identifier(_, value) => value.to_owned(),
			Self::Expression(value) => value.position()
		}
	}
}

impl Literal {
	pub fn position(&self) -> Position {
		match self {
			Self::Boolean(_, value) => value,
			Self::String(_, value) => value,
			Self::Number(_, value) => value
		}
		.to_owned()
	}
}

impl Hash for Literal {
	fn hash<H: hash::Hasher>(&self, state: &mut H) {
		match self {
			Self::Boolean(val, pos) => (val, pos).hash(state),
			Self::String(val, pos) => (val, pos).hash(state),
			Self::Number(val, pos) => (crate::utils::decode_f64(*val), pos).hash(state)
		}
	}
}

impl operators::BinaryOperator {
	pub fn get_precedence(&self) -> u8 {
		match self {
			Self::EqEq
			| Self::NotEq
			| Self::And
			| Self::Or
			| Self::Gt
			| Self::Lt
			| Self::Gte
			| Self::Lte => 1,
			Self::Add | Self::Subtract => 2,
			Self::Multiply | Self::Divide | Self::Modulo => 3,
			Self::ListAccess | Self::ObjectAccess => 4
		}
	}
}

//* From<T> *//

impl From<Term> for Expression {
	fn from(value: Term) -> Self {
		match value {
			Term::Expression(expression) => *expression,
			_ => Expression::Term(value)
		}
	}
}

impl_basic_conv!(from Object => Term as Object);
impl_basic_conv!(from List => Term as List);
impl_basic_conv!(from Call => Term as Call);
impl_basic_conv!(from Function => Term as Function);
impl_basic_conv!(from Literal => Term as Literal);
impl_basic_conv!(from Expression => Term as Expression { Box::new });
impl_basic_conv!(from Unary => Term as from { Expression::Unary });
impl_basic_conv!(from Binary => Term as from { Expression::Binary });

//* TryFrom<T> *//

impl TryFrom<String> for operators::BinaryOperator {
	type Error = String;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		match &value[..] {
			"+" => Ok(Self::Add),
			"-" => Ok(Self::Subtract),
			"/" => Ok(Self::Divide),
			"*" => Ok(Self::Multiply),
			"%" => Ok(Self::Modulo),
			"==" => Ok(Self::EqEq),
			"!=" => Ok(Self::NotEq),
			"and" => Ok(Self::And),
			"or" => Ok(Self::Or),
			">" => Ok(Self::Gt),
			"<" => Ok(Self::Lt),
			">=" => Ok(Self::Gte),
			"<=" => Ok(Self::Lte),
			"=>" => Ok(Self::ListAccess),
			"." => Ok(Self::ObjectAccess),

			op => Err(format!("Invalid binary operator '{op}'"))
		}
	}
}

impl TryFrom<String> for operators::UnaryOperator {
	type Error = String;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		match &value[..] {
			"-" => Ok(Self::Minus),
			"not" => Ok(Self::Not),

			op => Err(format!("Invalid unary operator '{op}'"))
		}
	}
}

//* Display *//

impl Display for operators::BinaryOperator {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Add => write!(f, "+"),
			Self::Subtract => write!(f, "-"),
			Self::Divide => write!(f, "/"),
			Self::Multiply => write!(f, "*"),
			Self::Modulo => write!(f, "%"),
			Self::EqEq => write!(f, "=="),
			Self::NotEq => write!(f, "!="),
			Self::And => write!(f, "and"),
			Self::Or => write!(f, "or"),
			Self::Gt => write!(f, ">"),
			Self::Lt => write!(f, "<"),
			Self::Gte => write!(f, ">="),
			Self::Lte => write!(f, "<="),
			Self::ListAccess => write!(f, "=>"),
			Self::ObjectAccess => write!(f, ".")
		}
	}
}

impl Display for operators::UnaryOperator {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Minus => write!(f, "-"),
			Self::Not => write!(f, "not")
		}
	}
}

impl Display for Expression {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Unary(value) => write!(f, "{value}"),
			Self::Binary(value) => write!(f, "{value}"),
			Self::Term(value) => write!(f, "{value}"),
			Self::Empty(_) => write!(f, "")
		}
	}
}

impl Display for Unary {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self.operator.0 {
			operators::UnaryOperator::Minus => write!(f, "{}{}", self.operator.0, self.operand),
			operators::UnaryOperator::Not => write!(f, "{} {}", self.operator.0, self.operand)
		}
	}
}

impl Display for Binary {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self.operator.0 {
			operators::BinaryOperator::ListAccess | operators::BinaryOperator::ObjectAccess => {
				write!(f, "{}{}{}", self.lhs, self.operator.0, self.rhs)
			}

			_ => write!(f, "{} {} {}", self.lhs, self.operator.0, self.rhs)
		}
	}
}

impl Display for Term {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Object(value) => write!(f, "{value}"),
			Self::List(value) => write!(f, "{value}"),
			Self::Call(value) => write!(f, "{value}"),
			Self::Function(value) => write!(f, "{value}"),
			Self::Literal(value) => write!(f, "{value}"),
			Self::Identifier(value, _) => write!(f, "{value}"),
			Self::Expression(value) => write!(f, "{value}")
		}
	}
}

impl Display for Object {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let formatted = self
			.0
			.iter()
			.map(|entry| format!("{} = {}", entry.name, entry.value))
			.collect::<Vec<String>>()
			.join(", ");

		write!(f, "{{ {formatted} }}")
	}
}

impl Display for List {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let formatted = self
			.0
			.iter()
			.map(|entry| entry.to_string())
			.collect::<Vec<String>>()
			.join(", ");

		write!(f, "[{formatted}]")
	}
}

impl Display for Call {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let function = self.function.to_string();

		let arguments = self
			.arguments
			.0
			.iter()
			.map(|arg| arg.to_string())
			.collect::<String>();

		write!(f, "{function}({arguments})")
	}
}

impl Display for Function {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let arguments = self
			.arguments
			.iter()
			.map(|(argument, _)| argument.to_owned())
			.collect::<Vec<_>>()
			.join(", ");

		write!(f, "func({arguments})")
	}
}

impl Display for Literal {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Boolean(value, _) => write!(f, "{value}"),
			Self::String(value, _) => write!(f, "{value}"),
			Self::Number(value, _) => write!(f, "{value}")
		}
	}
}
