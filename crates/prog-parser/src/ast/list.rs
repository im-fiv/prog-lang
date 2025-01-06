use anyhow::Result;

use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseStream, Span, Position};

#[derive(Debug, Clone, PartialEq)]
pub struct List<'inp> {
	pub _lb: token::LeftBracket<'inp>,
	pub items: Option<Box<Punctuated<'inp, Expr<'inp>, token::Comma<'inp>>>>,
	pub _rb: token::RightBracket<'inp>
}

impl ASTNode for List<'_> {
	fn span(&self) -> Span {
		let start = self._lb.start();
		let end = self._rb.end();

		let source = self._lb.source();
		let position = Position::new(start, end);

		Span::new(source, position)
	}
}

impl<'inp> Parse<'inp> for List<'inp> {
	fn parse(input: &ParseStream<'inp>) -> Result<Self> {
		let _lb = input.parse::<token::LeftBracket>()?;
		let items = input
			.try_parse::<Punctuated<'inp, Expr, token::Comma>>()
			.map(Box::new)
			.ok();
		let _rb = input.parse::<token::RightBracket>()?;

		Ok(Self {
			_lb,
			items,
			_rb
		})
	}
}
