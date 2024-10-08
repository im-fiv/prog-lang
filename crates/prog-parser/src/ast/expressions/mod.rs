mod impls;

use super::{Position, Statement};

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Expression {
	Unary(Unary),
	Binary(Binary),
	Term(Term),
	Empty(Option<Position>)
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Unary {
	pub operator: (operators::UnaryOperator, Position),
	pub operand: Term,
	pub position: Position
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Binary {
	pub lhs: Term,
	pub operator: (operators::BinaryOperator, Position),
	pub rhs: Term,
	pub position: Position
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Term {
	Extern(Extern),
	Object(Object),
	List(List),
	Call(Call),
	Function(Function),
	Literal(Literal),
	Identifier(String, Position),
	Expression(Box<Expression>)
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Extern(pub Box<Expression>, pub Position);

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Object(pub Vec<ObjectEntry>, pub Position);

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct ObjectEntry {
	pub name: String,
	pub value: Expression,
	pub position: Position
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct List(pub Vec<Expression>, pub Position);

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Call {
	pub arguments: (Vec<Expression>, Position),
	pub function: Box<Expression>,
	pub position: Position
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Function {
	pub arguments: Vec<(String, Position)>,
	pub statements: Vec<Statement>,
	pub position: Position
}

// Note: `Hash` is implemented in the `impls` module
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
	Boolean(bool, Position),
	String(String, Position),
	Number(f64, Position)
}

pub mod operators {
	#[derive(Debug, Clone, Copy, PartialEq, Hash)]
	#[cfg_attr(feature = "serde", derive(serde::Serialize))]
	pub enum BinaryOperator {
		Add,
		Subtract,
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

	#[derive(Debug, Clone, Copy, PartialEq, Hash)]
	#[cfg_attr(feature = "serde", derive(serde::Serialize))]
	pub enum UnaryOperator {
		Minus,
		Not
	}
}
