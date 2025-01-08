mod stmts;
mod binary_expr;
mod call;
mod expr;
mod ext;
mod field_acc;
mod func;
mod ident;
mod index_acc;
mod list;
mod lit;
mod obj;
mod punctuated;
mod term;
mod unary_expr;

pub use binary_expr::*;
pub use call::*;
pub use expr::*;
pub use ext::*;
pub use field_acc::*;
pub use func::*;
pub use ident::*;
pub use index_acc::*;
pub use list::*;
pub use lit::*;
pub use obj::*;
pub use punctuated::*;
pub use stmts::*;
pub use term::*;
pub use unary_expr::*;

use crate::{ASTNode, Parse, ParseResult, ParseStream, Span, Position};

macro_rules! op_to_token {
	($op:ident : $kind:ident => $token:ident) => {
		impl<'src> TryInto<$crate::token::$token<'src>> for $op<'src> {
			type Error = $crate::ParseError;

			fn try_into(self) -> ::std::result::Result<$crate::token::$token<'src>, Self::Error> {
				match self.kind {
					$kind::$token => Ok($crate::token::$token::new(self.span)),

					v => {
						Err($crate::ParseError::new(
							self.span.source().to_owned(),
							self.span.file().to_owned(),
							self.span.position(),
							$crate::ParseErrorKind::Internal($crate::error::Internal(
								format!(
									"Token of type `{:?}` cannot be converted to that of `{}`",
									v,
									stringify!($token)
								)
							))
						))
					}
				}
			}
		}
	};
}

pub(crate) use op_to_token;

#[derive(Debug, Clone, PartialEq)]
pub struct Program<'src> {
	pub stmts: Vec<Stmt<'src>>
}

impl ASTNode for Program<'_> {
	fn span(&self) -> Span {
		assert!(
			!self.stmts.is_empty(),
			"Could not get program's span as it is empty"
		);

		let first = self.stmts.first().unwrap().span();

		let start = first.position().start();
		let end = self.stmts.last().unwrap().end();

		let source = first.source();
		let file = first.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'src> Parse<'src> for Program<'src> {
	fn parse(input: &ParseStream<'src>) -> ParseResult<Self> {
		let mut stmts = vec![];

		while input.peek().is_some() {
			let stmt = input.parse::<Stmt>()?;
			stmts.push(stmt);
		}

		Ok(Self { stmts })
	}
}
