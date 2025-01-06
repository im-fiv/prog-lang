use anyhow::Result;

use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct Obj<'inp> {
	pub _lb: token::LeftBrace<'inp>,
	pub fields: Option<Box<Punctuated<'inp, ObjField<'inp>, token::Comma<'inp>>>>,
	pub _rb: token::RightBrace<'inp>
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjField<'inp> {
	pub name: token::Ident<'inp>,
	pub _eq: token::Eq<'inp>,
	pub value: Expr<'inp>
}

impl ASTNode for Obj<'_> {
	fn span(&self) -> Span {
		let start = self._lb.start();
		let end = self._rb.end();

		let source = self._lb.source();
		let position = Position::new(start, end);

		Span::new(source, position)
	}
}

impl ASTNode for ObjField<'_> {
	fn span(&self) -> Span {
		let start = self.name.start();
		let end = self.value.end();

		let source = self.name.source();
		let position = Position::new(start, end);

		Span::new(source, position)
	}
}

impl<'inp> Parse<'inp> for Obj<'inp> {
	fn parse(input: &ParseStream<'inp>) -> Result<Self> {
		let _lb = input.parse::<token::LeftBrace>()?;
		let fields = input
			.try_parse::<Punctuated<'inp, ObjField, token::Comma>>()
			.map(Box::new)
			.ok();
		let _rb = input.parse::<token::RightBrace>()?;

		Ok(Self { _lb, fields, _rb })
	}
}

impl<'inp> Parse<'inp> for ObjField<'inp> {
	fn parse(input: &ParseStream<'inp>) -> Result<Self> {
		let name = input.parse::<token::Ident>()?;
		let _eq = input.parse::<token::Eq>()?;
		let value = input.parse::<Expr>()?;

		Ok(Self { name, _eq, value })
	}
}
