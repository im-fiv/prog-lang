use std::fmt::Display;
use expressions::*;

pub use expressions::Expression;

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

	If {
		condition: Expression,
		statements: Vec<Statement>,
		elseif_branches: Vec<ConditionBranch>,
		else_branch: Option<ConditionBranch>
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
		Call(Call),
		Function(Function),
		Literal(Literal),
		Identifier(String),
		Expression(Box<Expression>)
	}

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
			Lte
		}

		#[derive(Debug, Clone, Copy, PartialEq)]
		pub enum UnaryOperator {
			Minus,
			Not
		}
	}
}

// implementations
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
			"&&" => Ok(Self::And),
			"||" => Ok(Self::Or),
			">" => Ok(Self::Gt),
			"<" => Ok(Self::Lt),
			">=" => Ok(Self::Gte),
			"<=" => Ok(Self::Lte),

			op => Err(format!("Invalid binary operator '{op}'"))
		}
	}
}

impl TryFrom<String> for operators::UnaryOperator {
	type Error = String;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		match &value[..] {
			"-" => Ok(Self::Minus),
			"!" => Ok(Self::Not),

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
			Self::And => write!(f, "&&"),
			Self::Or => write!(f, "||"),
			Self::Gt => write!(f, ">"),
			Self::Lt => write!(f, "<"),
			Self::Gte => write!(f, ">="),
			Self::Lte => write!(f, "<="),
		}
	}
}

impl Display for operators::UnaryOperator {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Minus => write!(f, "-"),
			Self::Not => write!(f, "!")
		}
	}
}