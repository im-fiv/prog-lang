use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseResult, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct FieldAcc<'inp> {
	pub object: Box<Term<'inp>>,
	pub _dot: token::Dot<'inp>,
	pub field: Ident<'inp>
}

impl<'inp> FieldAcc<'inp> {
	pub fn parse_with_object(
		input: &ParseStream<'inp>,
		object: Box<Term<'inp>>
	) -> ParseResult<Self> {
		let _dot = input.parse::<token::Dot>()?;
		let field = input.parse::<Ident>()?;

		Ok(Self {
			object,
			_dot,
			field
		})
	}
}

impl ASTNode for FieldAcc<'_> {
	fn span(&self) -> Span {
		let start = self.object.start();
		let end = self.field.end();

		let source = self.object.source();
		let file = self.object.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'inp> Parse<'inp> for FieldAcc<'inp> {
	fn parse(input: &ParseStream<'inp>) -> ParseResult<Self> {
		// To support chained operations or complex field access expressions
		// we have to rely on `Term`'s implementation
		Term::parse_variant::<Self>(input)
	}
}
