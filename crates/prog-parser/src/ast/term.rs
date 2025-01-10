use prog_lexer::TokenKind;

use crate::ast::*;
use crate::{error, ASTNode, Parse, ParseError, ParseErrorKind, ParseResult, ParseStream, Span};

#[derive(Debug, Clone, PartialEq, prog_macros::VariantUnwrap)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Term<'src> {
	// Wrapping terms
	Expr(Box<Expr<'src>>),
	ParenExpr(ParenExpr<'src>),

	// Regular terms
	Lit(Lit<'src>),
	Ident(Ident<'src>),
	Func(Box<Func<'src>>),
	List(List<'src>),
	Obj(Obj<'src>),
	Extern(Extern<'src>),

	// Lookahead terms
	Call(Call<'src>),
	IndexAcc(IndexAcc<'src>),
	FieldAcc(FieldAcc<'src>)
}

impl<'src> Term<'src> {
	pub fn parse_variant<T>(input: &ParseStream<'src>) -> ParseResult<T>
	where
		Self: TryInto<T>
	{
		let term = input.parse::<Self>()?;

		let err = ParseError::new(
			term.span(),
			ParseErrorKind::Internal(error::Internal(format!(
				"conversion of `{}` to variant `{}` failed",
				std::any::type_name::<Self>(),
				std::any::type_name::<T>()
			)))
		);

		term.try_into().map_err(|_| err)
	}
}

impl<'src> ASTNode<'src> for Term<'src> {
	fn span<'a>(&'a self) -> Span<'src> {
		match self {
			Self::Expr(t) => t.as_ref() as &dyn ASTNode,
			Self::ParenExpr(t) => t as &dyn ASTNode,
			Self::Lit(t) => t as &dyn ASTNode,
			Self::Ident(t) => t as &dyn ASTNode,
			Self::Func(t) => t.as_ref() as &dyn ASTNode,
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

impl<'src> Parse<'src> for Term<'src> {
	fn parse(input: &ParseStream<'src>) -> ParseResult<Self> {
		use TokenKind as T;

		let token = input.expect_peek()?;

		let mut term = match token.kind() {
			T::LeftParen => input.parse::<ParenExpr>().map(Self::ParenExpr)?,

			T::Number | T::True | T::False | T::String | T::None => {
				input.parse::<Lit>().map(Self::Lit)?
			}

			T::Ident => input.parse::<Ident>().map(Self::Ident)?,

			T::Func => input.parse::<Func>().map(|f| Self::Func(Box::new(f)))?,
			T::LeftBracket => input.parse::<List>().map(Self::List)?,
			T::LeftBrace => input.parse::<Obj>().map(Self::Obj)?,
			T::Extern => input.parse::<Extern>().map(Self::Extern)?,

			t => {
				return Err(ParseError::new(
					token.span(),
					ParseErrorKind::Internal(error::Internal(format!("unsupported term `{t:?}`")))
				))
			}
		};

		while let Some(token) = input.peek() {
			match token.kind() {
				T::LeftParen => {
					term = Call::parse_with_func(input, Box::new(term)).map(Self::Call)?;
				}

				T::LeftBracket => {
					term = IndexAcc::parse_with_list(input, Box::new(term)).map(Self::IndexAcc)?;
				}

				T::Dot => {
					term =
						FieldAcc::parse_with_object(input, Box::new(term)).map(Self::FieldAcc)?;
				}

				_ => break
			}
		}

		Ok(term)
	}
}
