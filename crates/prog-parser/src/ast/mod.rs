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
mod self_kw;
mod term;
mod stmts;
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
pub use self_kw::SelfKw;
pub use stmts::*;
pub use term::*;
pub use unary_expr::*;

use std::rc::Rc;

use crate::{ASTNode, Parse, ParseResult, ParseStream, Position, Span};

macro_rules! op_to_token {
	($op:ident : $kind:ident => $token:ident) => {
		impl<'src> TryInto<$crate::token::$token<'src>> for $op<'src> {
			type Error = $crate::ParseError<'src>;

			fn try_into(self) -> ::std::result::Result<$crate::token::$token<'src>, Self::Error> {
				match self.kind {
					$kind::$token => Ok($crate::token::$token::new(self.span)),

					v => {
						Err($crate::ParseError::new(
							self.span,
							$crate::ParseErrorKind::Internal($crate::error::Internal(format!(
								"token of type `{}` cannot be converted to that of `{}`",
								v,
								stringify!($token)
							)))
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
	pub stmts: Rc<[Stmt<'src>]>
}

impl<'src> ASTNode<'src> for Program<'src> {
	fn span<'a>(&'a self) -> Span<'src> {
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
	fn parse(input: &ParseStream<'src, '_>) -> ParseResult<'src, Self> {
		let mut stmts = vec![];

		while input.peek().is_some() {
			let stmt = input.parse::<Stmt>()?;
			stmts.push(stmt);
		}

		Ok(Self {
			stmts: stmts.into()
		})
	}
}

#[cfg(feature = "serde")]
impl serde::Serialize for Program<'_> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer
	{
		use serde::ser::SerializeStruct;

		let mut s = serializer.serialize_struct("Program", 1)?;
		s.serialize_field("stmts", self.stmts.as_ref())?;
		s.end()
	}
}
