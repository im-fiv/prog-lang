use std::fmt::Display;

pub use expressions::Expression;
use expressions::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
	pub statements: Vec<Statement>
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
	VariableDefine {
		name: String,
		value: Option<Expression>
	},

	VariableAssign {
		name: String,
		value: Expression
	},

	DoBlock(Vec<Statement>),
	Return(Option<Expression>),
	Call(Call),

	WhileLoop {
		condition: Expression,
		statements: Vec<Statement>
	},

	Break,
	Continue,

	If {
		condition: Expression,
		statements: Vec<Statement>,
		elseif_branches: Vec<ConditionBranch>,
		else_branch: Option<ConditionBranch>
	},

	ExpressionAssign {
		expression: Expression,
		value: Expression
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionBranch {
	pub condition: Expression,
	pub statements: Vec<Statement>
}

pub mod expressions {
	#[derive(Debug, Clone, PartialEq)]
	pub enum Expression {
		Unary(Unary),
		Binary(Binary),
		Term(Term),
		Empty
	}

	#[derive(Debug, Clone, PartialEq)]
	pub struct Unary {
		pub operator: operators::UnaryOperator,
		pub operand: Term
	}

	#[derive(Debug, Clone, PartialEq)]
	pub struct Binary {
		pub lhs: Term,
		pub operator: operators::BinaryOperator,
		pub rhs: Term
	}

	#[derive(Debug, Clone, PartialEq)]
	pub enum Term {
		Object(Object),
		List(List),
		Call(Call),
		Function(Function),
		Literal(Literal),
		Identifier(String),
		Expression(Box<Expression>)
	}

	#[derive(Debug, Clone, PartialEq)]
	pub struct Object(
		pub Vec<(String, Expression)>
	);

	#[derive(Debug, Clone, PartialEq)]
	pub struct List(
		pub Vec<Expression>
	);

	#[derive(Debug, Clone, PartialEq)]
	pub struct Call {
		pub arguments: Vec<Expression>,
		pub function: Box<Expression>
	}

	#[derive(Debug, Clone, PartialEq)]
	pub struct Function {
		pub arguments: Vec<String>,
		pub statements: Vec<super::Statement>
	}

	#[derive(Debug, Clone, PartialEq)]
	pub enum Literal {
		Boolean(bool),
		String(String),
		Number(f64)
	}

	pub mod operators {
		#[derive(Debug, Clone, Copy, PartialEq)]
		pub enum BinaryOperator {
			Plus,
			Minus,
			Divide,
			Multiply,
			Modulo,
			EqEq,
			NotEq,
			And,
			Or,
			Gt,
			Lt,
			Gte,
			Lte,
			ListAccess,
			ObjectAccess
		}

		#[derive(Debug, Clone, Copy, PartialEq)]
		pub enum UnaryOperator {
			Minus,
			Not
		}
	}
}

// Implementations
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

impl_basic_conv!(from Object => Term as Object);
impl_basic_conv!(from List => Term as List);
impl_basic_conv!(from Function => Term as Function);
impl_basic_conv!(from Literal => Term as Literal);
impl_basic_conv!(from Expression => Term as Expression { Box::new });
impl_basic_conv!(from Unary => Term as from { Expression::Unary });
impl_basic_conv!(from Binary => Term as from { Expression::Binary });

impl From<Term> for Expression {
	fn from(value: Term) -> Self {
		match value {
			Term::Expression(expression) => *expression,
			_ => Expression::Term(value)
		}
	}
}

impl_basic_conv!(from Call => Statement as Call);
impl_basic_conv!(from Call => Term as Call);

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

impl Display for expressions::Unary {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self.operator {
			operators::UnaryOperator::Minus => write!(f, "{}{}", self.operator, self.operand),
			operators::UnaryOperator::Not => write!(f, "{} {}", self.operator, self.operand)
		}
	}
}

impl Display for expressions::Binary {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self.operator {
			operators::BinaryOperator::ListAccess |
			operators::BinaryOperator::ObjectAccess => write!(f, "{}{}{}", self.lhs, self.operator, self.rhs),

			_ => write!(f, "{} {} {}", self.lhs, self.operator, self.rhs)
		}
	}
}

impl Display for expressions::Term {
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

impl Display for expressions::Object {
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

impl Display for expressions::List {
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

impl Display for expressions::Call {
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

impl Display for expressions::Function {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let arguments = self.arguments.join(", ");
		write!(f, "func({arguments})")
	}
}
impl Display for expressions::Literal {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Boolean(value) => write!(f, "{value}"),
			Self::String(value) => write!(f, "{value}"),
			Self::Number(value) => write!(f, "{value}")
		}
	}
}