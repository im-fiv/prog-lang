use anyhow::Result;

use crate::ast::*;
use crate::{token, ASTNode, Parse, ParsePrecedence, ParseStream, Position, Span};

#[derive(Debug)]
pub enum Expr<'inp> {
	Binary(BinaryExpr<'inp>),
	Unary(UnaryExpr<'inp>),
	Term(Term<'inp>)
}

impl ASTNode<'_> for Expr<'_> {
	fn span(&self) -> Span {
		// TODO
		todo!()
	}
}

impl<'inp> Parse<'inp> for Expr<'inp> {
	fn parse(input: &'inp ParseStream<'inp>) -> Result<Self> { Self::parse_precedence(input, 0) }
}

impl<'inp> ParsePrecedence<'inp> for Expr<'inp> {
	fn parse_precedence(input: &'inp ParseStream<'inp>, precedence: u8) -> Result<Self> {
		use prog_lexer::TokenKind;

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

			left = match op_token.kind {
				BinaryOpKind::LeftBracket => {
					let list = Box::new(match left {
						Expr::Term(t) => t,
						_ => todo!("error handling")
					});

					let _lb = op_token.try_into().unwrap();
					let expr = Self::parse_precedence(input, 0)?;
					let _rb = input.parse::<token::RightBracket>()?;

					Expr::Term(Term::IndexAcc(IndexAcc {
						list,
						_lb,
						index: Box::new(expr),
						_rb
					}))
				}

				BinaryOpKind::Dot => {
					let object = Box::new(match left {
						Expr::Term(t) => t,
						_ => todo!("error handling")
					});

					let _dot = op_token.try_into().unwrap();
					let field = input.parse::<token::Ident>()?;

					Expr::Term(Term::FieldAcc(FieldAcc {
						object,
						_dot,
						field
					}))
				}

				_ => {
					let right = Self::parse_precedence(input, right_binding_power)?;

					let lhs = match left {
						Expr::Term(t) => t,
						e => Term::Expr(Box::new(e))
					};

					let rhs = match right {
						Expr::Term(t) => t,
						e => Term::Expr(Box::new(e))
					};

					Expr::Binary(BinaryExpr {
						lhs,
						op: op_token,
						rhs
					})
				}
			};
		}

		Ok(left)
	}
}

#[derive(Debug)]
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
	fn parse(input: &'inp ParseStream<'inp>) -> Result<Self> {
		let _lp = input.parse::<token::LeftParen>()?;
		let expr = Box::new(Expr::parse_precedence(input, 0)?);
		let _rp = input.parse::<token::RightParen>()?;

		Ok(Self { _lp, expr, _rp })
	}
}
