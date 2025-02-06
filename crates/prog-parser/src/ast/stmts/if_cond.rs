use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseResult, ParseStream, Position, Span};

use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct If<'src> {
	pub _if: token::If<'src>,
	pub cond: Expr<'src>,
	pub _then: token::Then<'src>,
	pub stmts: Rc<[Stmt<'src>]>,
	pub b_elifs: Rc<[ElseIf<'src>]>,
	pub b_else: Option<Else<'src>>,
	pub _end: token::End<'src>
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ElseIf<'src> {
	pub _elseif: token::ElseIf<'src>,
	pub cond: Expr<'src>,
	pub _then: token::Then<'src>,
	pub stmts: Rc<[Stmt<'src>]>
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Else<'src> {
	pub _else: token::Else<'src>,
	pub stmts: Rc<[Stmt<'src>]>
}

impl<'src> ASTNode<'src> for If<'src> {
	fn span<'a>(&'a self) -> Span<'src> {
		let start = self._if.start();
		let end = self._end.end();

		let source = self._if.source();
		let file = self._if.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'src> ASTNode<'src> for ElseIf<'src> {
	fn span<'a>(&'a self) -> Span<'src> {
		let start = self._elseif.start();
		let end = match self.stmts.last() {
			Some(stmt) => stmt.end(),
			None => self._then.end()
		};

		let source = self._elseif.source();
		let file = self._elseif.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'src> ASTNode<'src> for Else<'src> {
	fn span<'a>(&'a self) -> Span<'src> {
		let start = self._else.start();
		let end = match self.stmts.last() {
			Some(stmt) => stmt.end(),
			None => self._else.end()
		};

		let source = self._else.source();
		let file = self._else.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'src> Parse<'src> for If<'src> {
	fn parse(input: &ParseStream<'src, '_>) -> ParseResult<'src, Self> {
		let _if = input.parse::<token::If>()?;
		let cond = input.parse::<Expr>()?;
		let _then = input.parse::<token::Then>()?;
		let mut stmts = vec![];

		while let Ok(stmt) = input.try_parse::<Stmt>() {
			stmts.push(stmt);
		}

		if let Ok(_end) = input.try_parse::<token::End>() {
			return Ok(Self {
				_if,
				cond,
				_then,
				stmts: stmts.into(),
				b_elifs: vec![].into(),
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
			stmts: stmts.into(),
			b_elifs: b_elifs.into(),
			b_else,
			_end
		})
	}
}

impl<'src> Parse<'src> for ElseIf<'src> {
	fn parse(input: &ParseStream<'src, '_>) -> ParseResult<'src, Self> {
		let _elseif = input.parse::<token::ElseIf>()?;
		let cond = input.parse::<Expr>()?;
		let _then = input.parse::<token::Then>()?;
		let mut stmts = vec![];

		while let Ok(stmt) = input.try_parse::<Stmt>() {
			stmts.push(stmt);
		}

		Ok(Self {
			_elseif,
			cond,
			_then,
			stmts: stmts.into()
		})
	}
}

impl<'src> Parse<'src> for Else<'src> {
	fn parse(input: &ParseStream<'src, '_>) -> ParseResult<'src, Self> {
		let _else = input.parse::<token::Else>()?;
		let mut stmts = vec![];

		while let Ok(stmt) = input.try_parse::<Stmt>() {
			stmts.push(stmt);
		}

		Ok(Self {
			_else,
			stmts: stmts.into()
		})
	}
}
