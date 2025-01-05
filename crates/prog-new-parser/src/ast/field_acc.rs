use anyhow::Result;

use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct FieldAcc<'inp> {
	pub object: Box<Term<'inp>>,
	pub _dot: token::Dot<'inp>,
	pub field: token::Ident<'inp>
}

impl<'inp> ASTNode<'inp> for FieldAcc<'inp> {
	fn span(&'inp self) -> prog_utils::pretty_errors::Span<'inp> {
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
		let _dot = input.parse::<token::Dot>()?;
		let field = input.parse::<token::Ident>()?;

		Ok(Self {
			object,
			_dot,
			field
		})
	}
}
