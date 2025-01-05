use anyhow::Result;

use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct VarAssign<'inp> {
	pub name: token::Ident<'inp>,
	pub _eq: token::Eq<'inp>,
	pub value: Expr<'inp>
}

impl ASTNode for VarAssign<'_> {
	fn span(&self) -> Span {
		let start = self.name.start();
		let end = self.value.end();

		let source = self.name.source();
		let position = Position::new(start, end);

		Span::new(source, position)
	}
}

impl<'inp> Parse<'inp> for VarAssign<'inp> {
	fn parse(input: &ParseStream<'inp>) -> Result<Self> {
		let name = input.parse::<token::Ident>()?;
		let _eq = input.parse::<token::Eq>()?;
		let value = input.parse::<Expr>()?;

		Ok(Self {
			name,
			_eq,
			value
		})
	}
}
