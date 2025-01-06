use anyhow::Result;

use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct ExprAssign<'inp> {
	pub expr: Expr<'inp>,
	pub _eq: token::Eq<'inp>,
	pub value: Expr<'inp>
}

impl ASTNode for ExprAssign<'_> {
	fn span(&self) -> Span {
		let start = self.expr.start();
		let end = self.value.end();

		let source = self.expr.source();
		let position = Position::new(start, end);

		Span::new(source, position)
	}
}

impl<'inp> Parse<'inp> for ExprAssign<'inp> {
	fn parse(input: &ParseStream<'inp>) -> Result<Self> {
		let expr = input.parse::<Expr>()?;
		let _eq = input.parse::<token::Eq>()?;
		let value = input.parse::<Expr>()?;

		Ok(Self { expr, _eq, value })
	}
}
