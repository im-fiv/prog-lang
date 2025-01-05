use anyhow::Result;

use crate::{token, ASTNode, Parse, ParseStream, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct Break<'inp> {
	_break: token::Break<'inp>
}

#[derive(Debug, Clone, PartialEq)]
pub struct Continue<'inp> {
	_continue: token::Continue<'inp>
}

impl ASTNode for Break<'_> {
	fn span(&self) -> Span {
		self._break.span()
	}
}

impl<'inp> Parse<'inp> for Break<'inp> {
	fn parse(input: &ParseStream<'inp>) -> Result<Self> {
		let _break = input.parse::<token::Break>()?;

		Ok(Self {
			_break
		})
	}
}

impl ASTNode for Continue<'_> {
	fn span(&self) -> Span {
		self._continue.span()
	}
}

impl<'inp> Parse<'inp> for Continue<'inp> {
	fn parse(input: &ParseStream<'inp>) -> Result<Self> {
		let _continue = input.parse::<token::Continue>()?;

		Ok(Self {
			_continue
		})
	}
}
