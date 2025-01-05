use crate::ast::*;
use crate::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr<'inp> {
	pub op: UnaryOp<'inp>,
	pub operand: Term<'inp>
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnaryOp<'inp> {
	pub kind: UnaryOpKind,
	pub span: Span<'inp>
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOpKind {
	Minus,
	Not
}
