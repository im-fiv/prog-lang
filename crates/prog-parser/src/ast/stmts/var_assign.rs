use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseResult, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct VarAssign<'src> {
	pub name: Ident<'src>,
	pub _eq: token::Eq<'src>,
	pub value: Expr<'src>
}

impl ASTNode for VarAssign<'_> {
	fn span(&self) -> Span {
		let start = self.name.start();
		let end = self.value.end();

		let source = self.name.source();
		let file = self.name.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'src> Parse<'src> for VarAssign<'src> {
	fn parse(input: &ParseStream<'src>) -> ParseResult<Self> {
		let name = input.parse::<Ident>()?;
		let _eq = input.parse::<token::Eq>()?;
		let value = input.parse::<Expr>()?;

		Ok(Self { name, _eq, value })
	}
}
