use std::fmt::Display;
use super::*;

macro_rules! impl_basic_conv {
	(from $from:ty => $for:ty as $variant:ident $({ $preproc:path })?) => {
		impl From<$from> for $for {
			fn from(value: $from) -> Self {
				Self::$variant(
					$( $preproc )? (value)
				)
			}
		}
	};
}

//* Others *//

impl operators::BinaryOperator {
	pub fn get_precedence(&self) -> u8 {
		match self {
			Self::EqEq | Self::NotEq | Self::And | Self::Or | Self::Gt | Self::Lt | Self::Gte | Self::Lte => 1,
			Self::Plus | Self::Minus => 2,
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
impl_basic_conv!(from Function => Term as Function);
impl_basic_conv!(from Literal => Term as Literal);
impl_basic_conv!(from Expression => Term as Expression { Box::new });
impl_basic_conv!(from Unary => Term as from { Expression::Unary });
impl_basic_conv!(from Binary => Term as from { Expression::Binary });

impl_basic_conv!(from Call => Statement as Call);
impl_basic_conv!(from Call => Term as Call);

//* TryFrom<T> *//

impl TryFrom<String> for operators::BinaryOperator {
	type Error = String;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		match &value[..] {
			"+" => Ok(Self::Plus),
			"-" => Ok(Self::Minus),
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
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Plus => write!(f, "+"),
			Self::Minus => write!(f, "-"),
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
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Minus => write!(f, "-"),
			Self::Not => write!(f, "not")
		}
	}
}

impl Display for Expression {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Unary(value) => write!(f, "{value}"),
			Self::Binary(value) => write!(f, "{value}"),
			Self::Term(value) => write!(f, "{value}"),
			Self::Empty => write!(f, "")
		}
	}
}

impl Display for Unary {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self.operator {
			operators::UnaryOperator::Minus => write!(f, "{}{}", self.operator, self.operand),
			operators::UnaryOperator::Not => write!(f, "{} {}", self.operator, self.operand)
		}
	}
}

impl Display for Binary {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self.operator {
			operators::BinaryOperator::ListAccess |
			operators::BinaryOperator::ObjectAccess => write!(f, "{}{}{}", self.lhs, self.operator, self.rhs),

			_ => write!(f, "{} {} {}", self.lhs, self.operator, self.rhs)
		}
	}
}

impl Display for Term {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Object(value) => write!(f, "{value}"),
			Self::List(value) => write!(f, "{value}"),
			Self::Call(value) => write!(f, "{value}"),
			Self::Function(value) => write!(f, "{value}"),
			Self::Literal(value) => write!(f, "{value}"),
			Self::Identifier(value) => write!(f, "{value}"),
			Self::Expression(value) => write!(f, "{value}"),
		}
	}
}

impl Display for Object {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let formatted = self
			.0
			.iter()
			.map(|(name, value)| format!("{name} = {value}"))
			.collect::<Vec<String>>()
			.join(", ");

		write!(f, "{{ {formatted} }}")
	}
}

impl Display for List {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let function = self
			.function
			.to_string();

		let arguments = self
			.arguments
			.iter()
			.map(|argument| argument.to_string())
			.collect::<Vec<String>>()
			.join(", ");

		write!(f, "{function}({arguments})")
	}
}

impl Display for Function {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let arguments = self.arguments.join(", ");
		write!(f, "func({arguments})")
	}
}

impl Display for Literal {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Boolean(value) => write!(f, "{value}"),
			Self::String(value) => write!(f, "{value}"),
			Self::Number(value) => write!(f, "{value}")
		}
	}
}