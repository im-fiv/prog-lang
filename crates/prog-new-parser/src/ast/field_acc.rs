use anyhow::Result;

use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct FieldAcc<'inp> {
	pub object: Box<Term<'inp>>,
	pub _dot: token::Dot<'inp>,
	pub field: token::Ident<'inp>
}

impl<'inp> FieldAcc<'inp> {
	pub fn parse_with_object(
		input: &'_ ParseStream<'inp>,
		object: Box<Term<'inp>>
	) -> Result<Self> {
		let _dot = input.parse::<token::Dot>()?;
		let field = input.parse::<token::Ident>()?;

		Ok(Self {
			object,
			_dot,
			field
		})
	}
}

impl ASTNode for FieldAcc<'_> {
	fn span(&self) -> Span {
		let start = self.object.span().start();
		let end = self.field.span().end();

		let source = self.object.span().source();
		let position = Position::new(start, end);

		Span::new(source, position)
	}
}

impl<'inp> Parse<'inp> for FieldAcc<'inp> {
	fn parse(input: &'_ ParseStream<'inp>) -> Result<Self> {
		let object = Box::new(input.parse::<Term>()?);
		Self::parse_with_object(input, object)
	}
}
