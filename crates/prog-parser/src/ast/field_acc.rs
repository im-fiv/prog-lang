use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseResult, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FieldAcc<'src> {
	pub object: Box<Term<'src>>,
	pub _dot: token::Dot<'src>,
	pub field: Ident<'src>
}

impl<'src> FieldAcc<'src> {
	pub fn parse_with_object(
		input: &ParseStream<'src>,
		object: Box<Term<'src>>
	) -> ParseResult<'src, Self> {
		let _dot = input.parse::<token::Dot>()?;
		let field = input.parse::<Ident>()?;

		Ok(Self {
			object,
			_dot,
			field
		})
	}
}

impl<'src> ASTNode<'src> for FieldAcc<'src> {
	fn span<'a>(&'a self) -> Span<'src> {
		let start = self.object.start();
		let end = self.field.end();

		let source = self.object.source();
		let file = self.object.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'src> Parse<'src> for FieldAcc<'src> {
	fn parse(input: &ParseStream<'src>) -> ParseResult<'src, Self> {
		// To support chained operations or complex field access expressions
		// we have to rely on `Term`'s implementation
		Term::parse_variant::<Self>(input)
	}
}
