use prog_lexer::TokenKind;

use crate::ast::*;
use crate::{errors, ASTNode, Parse, ParseError, ParseErrorKind, ParseResult, ParseStream, Span};

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
	pub fn parse_variant<T>(input: &ParseStream<'inp>) -> ParseResult<T>
	where
		Self: TryInto<T>
	{
		let term = input.parse::<Self>()?;

		let source = term.span().source().to_owned();
		let file = term.span().file().to_owned();
		let position = term.span().position();

		term.try_into().map_err(|_| {
			ParseError::new(
				source,
				file,
				position,
				ParseErrorKind::Internal(errors::Internal(format!(
					"Conversion of `{}` to variant `{}` failed",
					std::any::type_name::<Self>(),
					std::any::type_name::<T>()
				)))
			)
		})
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
	fn parse(input: &ParseStream<'inp>) -> ParseResult<Self> {
		use TokenKind as T;

		let token = input.expect_peek()?;

		let mut term = match token.kind() {
			T::LeftParen => input.parse::<ParenExpr>().map(Self::ParenExpr)?,

			T::Number | T::True | T::False | T::String | T::None => {
				input.parse::<Lit>().map(Self::Lit)?
			}

			T::Ident => input.parse::<Ident>().map(Self::Ident)?,

			T::Func => input.parse::<Func>().map(Self::Func)?,
			T::LeftBracket => input.parse::<List>().map(Self::List)?,
			T::LeftBrace => input.parse::<Obj>().map(Self::Obj)?,
			T::Extern => input.parse::<Extern>().map(Self::Extern)?,

			t => {
				return Err(ParseError::new(
					token.span().source().to_owned(),
					token.span().file().to_owned(),
					token.span().position(),
					ParseErrorKind::Internal(errors::Internal(format!("unsupported term `{t:?}`")))
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
