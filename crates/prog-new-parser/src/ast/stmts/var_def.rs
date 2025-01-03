use anyhow::Result;

use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseStream, Position, Span, Token};

#[derive(Debug)]
pub struct VariableDefinition<'inp> {
	pub _def: token::Def<'inp>,
	pub name: token::Ident<'inp>,
	pub _eq: token::Eq<'inp>,
	pub value: Expr<'inp>
}

impl ASTNode<'_> for VariableDefinition<'_> {
	fn span(&self) -> Span {
		let start = Token::span(&self._def).start();
		let end = self.value.span().end();

		let source = Token::span(&self._def).source();
		let position = Position::new(start, end);

		Span::new(source, position)
	}
}

impl<'inp> Parse<'inp> for VariableDefinition<'inp> {
	fn parse(input: &'inp ParseStream<'inp>) -> Result<Self> {
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
