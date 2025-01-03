use crate::ast::*;
use crate::token;

#[derive(Debug)]
pub struct IndexAcc<'inp> {
	pub list: Box<Term<'inp>>,
	pub _lb: token::LeftBracket<'inp>,
	pub index: Box<Expr<'inp>>,
	pub _rb: token::RightBracket<'inp>
}
