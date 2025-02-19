use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseResult, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ExprAssign<'src> {
	IndexAssign(IndexAssign<'src>),
	FieldAssign(FieldAssign<'src>)
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct IndexAssign<'src> {
	pub acc: IndexAcc<'src>,
	pub _eq: token::Eq<'src>,
	pub value: Expr<'src>
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FieldAssign<'src> {
	pub acc: FieldAcc<'src>,
	pub _eq: token::Eq<'src>,
	pub value: Expr<'src>
}

impl<'src> ASTNode<'src> for ExprAssign<'src> {
	fn span<'a>(&'a self) -> Span<'src> {
		match self {
			Self::IndexAssign(a) => a as &dyn ASTNode,
			Self::FieldAssign(a) => a as &dyn ASTNode
		}
		.span()
	}
}

impl<'src> ASTNode<'src> for IndexAssign<'src> {
	fn span<'a>(&'a self) -> Span<'src> {
		let start = self.acc.start();
		let end = self.value.end();

		let source = self.acc.source();
		let file = self.acc.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'src> ASTNode<'src> for FieldAssign<'src> {
	fn span<'a>(&'a self) -> Span<'src> {
		let start = self.acc.start();
		let end = self.value.end();

		let source = self.acc.source();
		let file = self.acc.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'src> Parse<'src> for ExprAssign<'src> {
	fn parse(input: &ParseStream<'src, '_>) -> ParseResult<'src, Self> {
		input
			.try_parse::<IndexAssign>()
			.map(Self::IndexAssign)
			.or(input.try_parse::<FieldAssign>().map(Self::FieldAssign))
	}
}

impl<'src> Parse<'src> for IndexAssign<'src> {
	fn parse(input: &ParseStream<'src, '_>) -> ParseResult<'src, Self> {
		let acc = input.parse::<IndexAcc>()?;
		let _eq = input.parse::<token::Eq>()?;
		let value = input.parse::<Expr>()?;

		Ok(Self { acc, _eq, value })
	}
}

impl<'src> Parse<'src> for FieldAssign<'src> {
	fn parse(input: &ParseStream<'src, '_>) -> ParseResult<'src, Self> {
		let acc = input.parse::<FieldAcc>()?;
		let _eq = input.parse::<token::Eq>()?;
		let value = input.parse::<Expr>()?;

		Ok(Self { acc, _eq, value })
	}
}
