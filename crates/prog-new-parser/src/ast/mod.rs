mod stmts;
mod binary_expr;
mod call;
mod expr;
mod field_acc;
mod index_acc;
mod lit;
mod punctuated;
mod term;
mod unary_expr;

pub use binary_expr::*;
pub use call::*;
pub use expr::*;
pub use field_acc::*;
pub use index_acc::*;
pub use lit::*;
pub use punctuated::*;
pub use stmts::*;
pub use term::*;
pub use unary_expr::*;

macro_rules! op_to_token {
	($op:ident : $kind:ident => $token:ident) => {
		impl<'inp> TryInto<$crate::token::$token<'inp>> for $op<'inp> {
			type Error = ::anyhow::Error;

			fn try_into(self) -> Result<$crate::token::$token<'inp>, Self::Error> {
				match self.kind {
					$kind::$token => Ok($crate::token::$token { span: self.span }),

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
