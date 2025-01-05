use anyhow::Result;

use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct VarDefine<'inp> {
	pub _def: token::Def<'inp>,
	pub name: token::Ident<'inp>,
	pub _eq: token::Eq<'inp>,
	pub value: Expr<'inp>
}

impl ASTNode for VarDefine<'_> {
	fn span(&self) -> Span {
		let start = self._def.start();
		let end = self.value.end();

		let source = self._def.source();
		let position = Position::new(start, end);

		Span::new(source, position)
	}
}

impl<'inp> Parse<'inp> for VarDefine<'inp> {
	fn parse(input: &ParseStream<'inp>) -> Result<Self> {
		let _def = input.parse::<token::Def>()?;
		let name = input.parse::<token::Ident>()?;
		let _eq = input.parse::<token::Eq>()?;
		let value = input.parse::<Expr>()?;

		Ok(Self {
			_def,
			name,
			_eq,
			value
		})
	}
}
