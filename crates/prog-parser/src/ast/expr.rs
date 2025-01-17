use prog_lexer::TokenKind;

use crate::ast::*;
use crate::{token, ASTNode, Parse, ParsePrecedence, ParseResult, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Expr<'src> {
	Binary(BinaryExpr<'src>),
	Unary(UnaryExpr<'src>),
	Term(Term<'src>)
}

impl<'src> ASTNode<'src> for Expr<'src> {
	fn span<'a>(&'a self) -> Span<'src> {
		match self {
			Self::Binary(e) => e as &dyn ASTNode,
			Self::Unary(e) => e as &dyn ASTNode,
			Self::Term(e) => e as &dyn ASTNode
		}
		.span()
	}
}

impl<'src> Parse<'src> for Expr<'src> {
	fn parse(input: &ParseStream<'src>) -> ParseResult<'src, Self> {
		Self::parse_precedence(input, 0)
	}
}

impl<'src> ParsePrecedence<'src> for Expr<'src> {
	fn parse_precedence(input: &ParseStream<'src>, precedence: u8) -> ParseResult<'src, Self> {
		use TokenKind as T;

		let mut left = Self::Term(input.parse::<Term>()?);

		while let Some(token) = input.peek() {
			let infix_binding_power = match token.kind() {
				T::Plus | T::Minus => (1, 2),

				T::Asterisk | T::Slash => (3, 4),

				T::Dot | T::LeftBracket => (5, 6),

				T::EqEq | T::Gt | T::Lt | T::Gte | T::Lte => (1, 2),

				T::And => (3, 2),
				T::Or => (1, 1),

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
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ParenExpr<'src> {
	pub _lp: token::LeftParen<'src>,
	pub expr: Box<Expr<'src>>,
	pub _rp: token::RightParen<'src>
}

impl<'src> ASTNode<'src> for ParenExpr<'src> {
	fn span<'a>(&'a self) -> Span<'src> {
		let start = self._lp.start();
		let end = self._rp.end();

		let source = self._lp.source();
		let file = self._lp.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'src> Parse<'src> for ParenExpr<'src> {
	fn parse(input: &ParseStream<'src>) -> ParseResult<'src, Self> {
		let _lp = input.parse::<token::LeftParen>()?;
		let expr = Box::new(Expr::parse_precedence(input, 0)?);
		let _rp = input.parse::<token::RightParen>()?;

		Ok(Self { _lp, expr, _rp })
	}
}
