use anyhow::Result;

use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct If<'inp> {
	pub _if: token::If<'inp>,
	pub cond: Expr<'inp>,
	pub _then: token::Then<'inp>,
	pub stmts: Vec<Statement<'inp>>,
	pub b_elifs: Vec<ElseIf<'inp>>,
	pub b_else: Option<Else<'inp>>,
	pub _end: token::End<'inp>
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElseIf<'inp> {
	pub _elseif: token::ElseIf<'inp>,
	pub cond: Expr<'inp>,
	pub _then: token::Then<'inp>,
	pub stmts: Vec<Statement<'inp>>
}

#[derive(Debug, Clone, PartialEq)]
pub struct Else<'inp> {
	pub _else: token::Else<'inp>,
	pub stmts: Vec<Statement<'inp>>
}

impl ASTNode for If<'_> {
	fn span(&self) -> Span {
		let start = self._if.start();
		let end = self._end.end();

		let source = self._if.source();
		let position = Position::new(start, end);

		Span::new(source, position)
	}
}

impl ASTNode for ElseIf<'_> {
	fn span(&self) -> Span {
		let start = self._elseif.start();
		let end = match self.stmts.last() {
			Some(stmt) => stmt.end(),
			None => self._then.end()
		};

		let source = self._elseif.source();
		let position = Position::new(start, end);

		Span::new(source, position)
	}
}

impl ASTNode for Else<'_> {
	fn span(&self) -> Span {
		let start = self._else.start();
		let end = match self.stmts.last() {
			Some(stmt) => stmt.end(),
			None => self._else.end()
		};

		let source = self._else.source();
		let position = Position::new(start, end);

		Span::new(source, position)
	}
}

impl<'inp> Parse<'inp> for If<'inp> {
	fn parse(input: &ParseStream<'inp>) -> Result<Self> {
		let _if = input.parse::<token::If>()?;
		let cond = input.parse::<Expr>()?;
		let _then = input.parse::<token::Then>()?;
		let mut stmts = vec![];

		while let Ok(stmt) = input.try_parse::<Statement>() {
			stmts.push(stmt);
		}

		if let Ok(_end) = input.try_parse::<token::End>() {
			return Ok(Self {
				_if,
				cond,
				_then,
				stmts,
				b_elifs: vec![],
				b_else: None,
				_end
			});
		}

		let mut b_elifs = vec![];
		let mut b_else = None;

		while let Ok(b) = input.try_parse::<ElseIf>() {
			b_elifs.push(b);
		}

		if let Ok(b) = input.try_parse::<Else>() {
			b_else = Some(b);
		}

		let _end = input.parse::<token::End>()?;

		Ok(Self {
			_if,
			cond,
			_then,
			stmts,
			b_elifs,
			b_else,
			_end
		})
	}
}

impl<'inp> Parse<'inp> for ElseIf<'inp> {
	fn parse(input: &ParseStream<'inp>) -> Result<Self> {
		let _elseif = input.parse::<token::ElseIf>()?;
		let cond = input.parse::<Expr>()?;
		let _then = input.parse::<token::Then>()?;
		let mut stmts = vec![];

		while let Ok(stmt) = input.try_parse::<Statement>() {
			stmts.push(stmt);
		}

		Ok(Self {
			_elseif,
			cond,
			_then,
			stmts
		})
	}
}

impl<'inp> Parse<'inp> for Else<'inp> {
	fn parse(input: &ParseStream<'inp>) -> Result<Self> {
		let _else = input.parse::<token::Else>()?;
		let mut stmts = vec![];

		while let Ok(stmt) = input.try_parse::<Statement>() {
			stmts.push(stmt);
		}

		Ok(Self { _else, stmts })
	}
}
