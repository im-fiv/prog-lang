use crate::ast::*;
use crate::token;

#[derive(Debug, Clone, PartialEq)]
pub struct FieldAcc<'inp> {
	pub object: Box<Term<'inp>>,
	pub _dot: token::Dot<'inp>,
	pub field: token::Ident<'inp>
}
