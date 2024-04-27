#[derive(Debug)]
pub struct Program {
	pub statements: Vec<statements::Statement>
}

pub mod statements {
    use super::expressions;

	#[derive(Debug)]
	pub enum Statement {
		VariableDefine(VariableDefine),
		VariableAssign(VariableAssign)
	}

	#[derive(Debug)]
	pub struct VariableDefine {
		pub name: String,
		pub value: Option<expressions::Expression>
	}

	#[derive(Debug)]
	pub struct VariableAssign {
		pub name: String,
		pub value: expressions::Expression
	}
}

pub mod expressions {
	#[derive(Debug)]
	pub enum Expression {
		Unary(Unary),
		Binary(Binary),
		Term(Term)
	}

	#[derive(Debug)]
	pub struct Unary {
		pub operator: operators::UnaryOperator,
		pub operand: Box<Expression>
	}

	#[derive(Debug)]
	pub struct Binary {
		pub lhs: Box<Expression>,
		pub operator: operators::BinaryOperator,
		pub rhs: Box<Expression>
	}

	#[derive(Debug)]
	pub enum Term {
		Literal(Literal),
		Identifier(String),
		Expression(Box<Expression>)
	}

	#[derive(Debug)]
	pub enum Literal {
		Boolean(bool),
		String(String),
		Number(f64)
	}

	pub mod operators {
		#[derive(Debug)]
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

		#[derive(Debug)]
		pub enum UnaryOperator {
			Minus,
			Not
		}
	}
}

// implementations
impl expressions::operators::BinaryOperator {
	pub fn get_precedence(&self) -> u8 {
		match self {
			Self::EqEq | Self::NotEq | Self::And | Self::Or | Self::Gt | Self::Lt | Self::Gte | Self::Lte => 1,
			Self::Plus | Self::Minus => 2,
			Self::Multiply | Self::Divide | Self::Modulo => 3,
		}
	}
}

impl TryFrom<String> for expressions::operators::BinaryOperator {
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

impl TryFrom<String> for expressions::operators::UnaryOperator {
	type Error = String;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		match &value[..] {
			"-" => Ok(Self::Minus),
			"!" => Ok(Self::Not),

			op => Err(format!("Invalid unary operator '{op}'"))
		}
	}
}