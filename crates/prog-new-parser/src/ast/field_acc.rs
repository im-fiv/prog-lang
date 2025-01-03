use crate::ast::*;
use crate::token;

#[derive(Debug)]
pub struct FieldAcc<'inp> {
	pub object: Box<Term<'inp>>,
	pub _dot: token::Dot<'inp>,
	pub field: token::Ident<'inp>
}
