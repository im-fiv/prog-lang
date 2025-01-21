use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseResult, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExprAssign<'src> {
	pub expr: Expr<'src>,
	pub _eq: token::Eq<'src>,
	pub value: Expr<'src>
}

impl<'src> ASTNode<'src> for ExprAssign<'src> {
	fn span<'a>(&'a self) -> Span<'src> {
		let start = self.expr.start();
		let end = self.value.end();

		let source = self.expr.source();
		let file = self.expr.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'src> Parse<'src> for ExprAssign<'src> {
	fn parse(input: &ParseStream<'src, '_>) -> ParseResult<'src, Self> {
		let expr = input.parse::<Expr>()?;
		let _eq = input.parse::<token::Eq>()?;
		let value = input.parse::<Expr>()?;

		Ok(Self { expr, _eq, value })
	}
}
