use anyhow::{bail, Result};
use prog_lexer::TokenKind;

use crate::ast::*;
use crate::{ASTNode, Parse, ParseStream, Span};

#[derive(Debug, Clone, PartialEq, prog_macros::VariantUnwrap)]
pub enum Term<'inp> {
	// Wrapping terms
	Expr(Box<Expr<'inp>>),
	ParenExpr(ParenExpr<'inp>),

	// Regular terms
	Lit(Lit<'inp>),
	Ident(Ident<'inp>),
	Func(Func<'inp>),
	List(List<'inp>),
	Obj(Obj<'inp>),
	Extern(Extern<'inp>),

	// Lookahead terms
	Call(Call<'inp>),
	IndexAcc(IndexAcc<'inp>),
	FieldAcc(FieldAcc<'inp>)
}

impl<'inp> Term<'inp> {
	/// Unlike the `Parse` implementation, does not parse more than it has to.
	///
	/// Useful when the parse call originates from `Term`'s variants to prevent
	/// `Term` from consuming the tokens that its variant was supposed to consume.
	pub fn parse_bounded(input: &ParseStream<'inp>, bounded: bool) -> Result<Self> {
		let token = input.expect_peek()?;

		let mut term = match token.kind() {
			TokenKind::LeftParen => input.parse::<ParenExpr>().map(Self::ParenExpr)?,

			TokenKind::Number | TokenKind::True | TokenKind::False | TokenKind::String => {
				input.parse::<Lit>().map(Self::Lit)?
			}

			TokenKind::Ident => input.parse::<Ident>().map(Self::Ident)?,

			TokenKind::Func => input.parse::<Func>().map(Self::Func)?,
			TokenKind::LeftBracket => input.parse::<List>().map(Self::List)?,
			TokenKind::LeftBrace => input.parse::<Obj>().map(Self::Obj)?,
			TokenKind::Extern => input.parse::<Extern>().map(Self::Extern)?,

			// TODO: proper error reporting
			t => bail!("unsupported term `{t:?}`")
		};

		if bounded {
			return Ok(term);
		}

		while let Some(token) = input.peek() {
			match token.kind() {
				TokenKind::LeftParen => {
					term = Call::parse_with_func(input, Box::new(term)).map(Self::Call)?;
				}

				TokenKind::LeftBracket => {
					term = IndexAcc::parse_with_list(input, Box::new(term)).map(Self::IndexAcc)?;
				}

				TokenKind::Dot => {
					term =
						FieldAcc::parse_with_object(input, Box::new(term)).map(Self::FieldAcc)?;
				}

				_ => break
			}
		}

		Ok(term)
	}

	pub fn parse_variant<T>(input: &ParseStream<'inp>) -> Result<T>
	where
		Self: TryInto<T>,
		<Self as TryInto<T>>::Error: std::fmt::Debug
	{
		let term = input.parse::<Self>()?;
		
		// We want this operation to panic if the conversion is invalid,
		// as this function's success should only be determined by parsing.
		Ok(term.try_into().unwrap())
	}
}

impl ASTNode for Term<'_> {
	fn span(&self) -> Span {
		match self {
			Self::Expr(t) => &**t as &dyn ASTNode,
			Self::ParenExpr(t) => t as &dyn ASTNode,
			Self::Lit(t) => t as &dyn ASTNode,
			Self::Ident(t) => t as &dyn ASTNode,
			Self::Func(t) => t as &dyn ASTNode,
			Self::List(t) => t as &dyn ASTNode,
			Self::Obj(t) => t as &dyn ASTNode,
			Self::Extern(t) => t as &dyn ASTNode,
			Self::Call(t) => t as &dyn ASTNode,
			Self::IndexAcc(t) => t as &dyn ASTNode,
			Self::FieldAcc(t) => t as &dyn ASTNode
		}
		.span()
	}
}

impl<'inp> Parse<'inp> for Term<'inp> {
	fn parse(input: &ParseStream<'inp>) -> Result<Self> { Self::parse_bounded(input, false) }
}
