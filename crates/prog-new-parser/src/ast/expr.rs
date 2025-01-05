use anyhow::Result;
use prog_lexer::TokenKind;

use crate::ast::*;
use crate::{token, ASTNode, Parse, ParsePrecedence, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr<'inp> {
	Binary(BinaryExpr<'inp>),
	Unary(UnaryExpr<'inp>),
	Term(Term<'inp>)
}

impl ASTNode<'_> for Expr<'_> {
	fn span(&self) -> Span {
		match self {
			Self::Binary(e) => e as &dyn ASTNode,
			Self::Unary(e) => e as &dyn ASTNode,
			Self::Term(e) => e as &dyn ASTNode
		}
		.span()
	}
}

impl<'inp> Parse<'inp> for Expr<'inp> {
	fn parse(input: &ParseStream<'inp>) -> Result<Self> { Self::parse_precedence(input, 0) }
}

impl<'inp> ParsePrecedence<'inp> for Expr<'inp> {
	fn parse_precedence(input: &ParseStream<'inp>, precedence: u8) -> Result<Self> {
		let mut left = Self::Term(input.parse::<Term>()?);

		while let Some(token) = input.peek() {
			let infix_binding_power = match token.kind() {
				TokenKind::Plus | TokenKind::Minus => (1, 2),

				TokenKind::Asterisk | TokenKind::Slash => (3, 4),

				TokenKind::Dot | TokenKind::LeftBracket => (5, 6),

				TokenKind::EqEq
				| TokenKind::Gt
				| TokenKind::Lt
				| TokenKind::Gte
				| TokenKind::Lte => (1, 2),

				_ => break
			};

			let (left_binding_power, right_binding_power) = infix_binding_power;

			if left_binding_power < precedence {
				break;
			}

			let op_token = input.parse::<BinaryOp>()?;

			let lhs = match left {
				Self::Term(t) => t,
				e => Term::Expr(Box::new(e))
			};

			let rhs = match Self::parse_precedence(input, right_binding_power)? {
				Self::Term(t) => t,
				e => Term::Expr(Box::new(e))
			};

			left = Self::Binary(BinaryExpr {
				lhs,
				op: op_token,
				rhs
			});
		}

		Ok(left)
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParenExpr<'inp> {
	pub _lp: token::LeftParen<'inp>,
	pub expr: Box<Expr<'inp>>,
	pub _rp: token::RightParen<'inp>
}

impl<'inp> ASTNode<'inp> for ParenExpr<'inp> {
	fn span(&'inp self) -> Span<'inp> {
		let start = ASTNode::span(&self._lp).start();
		let end = ASTNode::span(&self._rp).end();

		let source = ASTNode::span(&self._lp).source();
		let position = Position::new(start, end);

		Span::new(source, position)
	}
}

impl<'inp> Parse<'inp> for ParenExpr<'inp> {
	fn parse(input: &'_ ParseStream<'inp>) -> Result<Self> {
		let _lp = input.parse::<token::LeftParen>()?;
		let expr = Box::new(Expr::parse_precedence(input, 0)?);
		let _rp = input.parse::<token::RightParen>()?;

		Ok(Self { _lp, expr, _rp })
	}
}
