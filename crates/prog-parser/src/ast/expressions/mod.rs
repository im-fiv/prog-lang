mod impls;
use super::Statement;

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
	pub statements: Vec<Statement>
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