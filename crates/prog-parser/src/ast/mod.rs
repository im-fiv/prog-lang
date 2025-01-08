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

macro_rules! op_to_token {
	($op:ident : $kind:ident => $token:ident) => {
		impl<'src> TryInto<$crate::token::$token<'src>> for $op<'src> {
			type Error = ::anyhow::Error;

			fn try_into(self) -> ::std::result::Result<$crate::token::$token<'src>, Self::Error> {
				match self.kind {
					$kind::$token => Ok($crate::token::$token::new(self.span)),

					v => {
						Err(::anyhow::anyhow!(
							"Token of type `{:?}` cannot be converted to that of `{}`",
							v,
							stringify!($token)
						))
					}
				}
			}
		}
	};
}

pub(crate) use op_to_token;
