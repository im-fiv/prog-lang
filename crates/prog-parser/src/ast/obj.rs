use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseResult, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Obj<'src> {
	pub _lb: token::LeftBrace<'src>,
	pub fields: Option<Box<Punctuated<'src, ObjField<'src>, token::Comma<'src>>>>,
	pub _rb: token::RightBrace<'src>
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ObjField<'src> {
	pub name: Ident<'src>,
	pub _eq: token::Eq<'src>,
	pub value: Expr<'src>
}

impl ASTNode for Obj<'_> {
	fn span(&self) -> Span {
		let start = self._lb.start();
		let end = self._rb.end();

		let source = self._lb.source();
		let file = self._lb.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl ASTNode for ObjField<'_> {
	fn span(&self) -> Span {
		let start = self.name.start();
		let end = self.value.end();

		let source = self.name.source();
		let file = self.name.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'src> Parse<'src> for Obj<'src> {
	fn parse(input: &ParseStream<'src>) -> ParseResult<Self> {
		let _lb = input.parse::<token::LeftBrace>()?;
		let fields = input
			.try_parse::<Punctuated<'src, ObjField, token::Comma>>()
			.map(Box::new)
			.ok();
		let _rb = input.parse::<token::RightBrace>()?;

		Ok(Self { _lb, fields, _rb })
	}
}

impl<'src> Parse<'src> for ObjField<'src> {
	fn parse(input: &ParseStream<'src>) -> ParseResult<Self> {
		let name = input.parse::<Ident>()?;
		let _eq = input.parse::<token::Eq>()?;
		let value = input.parse::<Expr>()?;

		Ok(Self { name, _eq, value })
	}
}
