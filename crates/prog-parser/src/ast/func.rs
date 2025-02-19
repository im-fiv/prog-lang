use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseResult, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Func<'src> {
	pub _func: token::Func<'src>,
	pub _lp: token::LeftParen<'src>,
	pub args: FuncArgs<'src>,
	pub _rp: token::RightParen<'src>,
	pub block: DoBlock<'src>
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum FuncArgs<'src> {
	WithSelf {
		_self: SelfKw<'src>,
		args: Option<(
			token::Comma<'src>,
			Punctuated<'src, Ident<'src>, token::Comma<'src>>
		)>
	},

	WithoutSelf {
		args: Punctuated<'src, Ident<'src>, token::Comma<'src>>
	}
}

impl FuncArgs<'_> {
	pub fn is_empty(&self) -> bool {
		match self {
			Self::WithSelf { _self, .. } => false,
			Self::WithoutSelf { args } => args.is_empty()
		}
	}

	pub fn items(&self) -> Vec<&dyn ASTNode> {
		let mut items = vec![];

		match self {
			Self::WithSelf { _self, args } => {
				items.push(_self as &dyn ASTNode);

				if let Some((_, args)) = args {
					for item in args.items() {
						items.push(item as &dyn ASTNode);
					}
				}
			}

			Self::WithoutSelf { args } => {
				for item in args.items() {
					items.push(item as &dyn ASTNode);
				}
			}
		}

		items
	}
}

impl<'src> ASTNode<'src> for Func<'src> {
	fn span<'a>(&'a self) -> Span<'src> {
		let start = self._func.start();
		let end = self.block.end();

		let source = self._func.source();
		let file = self._func.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'src> ASTNode<'src> for FuncArgs<'src> {
	fn span<'a>(&'a self) -> Span<'src> {
		match self {
			Self::WithSelf { _self, args } => {
				let start = _self.start();
				let end = match args {
					Some((_, args)) if !args.is_empty() => args.end(),
					_ => _self.end()
				};

				let source = _self.source();
				let file = _self.file();
				let position = Position::new(start, end);

				Span::new(source, file, position)
			}

			Self::WithoutSelf { args } => args.span()
		}
	}
}

impl<'src> Parse<'src> for Func<'src> {
	fn parse(input: &ParseStream<'src, '_>) -> ParseResult<'src, Self> {
		let _func = input.parse::<token::Func>()?;
		let _lp = input.parse::<token::LeftParen>()?;
		let args = input.try_parse::<FuncArgs>()?;
		let _rp = input.parse::<token::RightParen>()?;
		let block = input.parse::<DoBlock>()?;

		Ok(Self {
			_func,
			_lp,
			args,
			_rp,
			block
		})
	}
}

impl<'src> Parse<'src> for FuncArgs<'src> {
	fn parse(input: &ParseStream<'src, '_>) -> ParseResult<'src, Self> {
		use prog_lexer::TokenKind;

		let self_arg = match input.try_parse::<SelfKw>() {
			Ok(_self) => {
				// If `self` is not the only argument, require a comma before the rest of the arguments
				if input.peek_matches(TokenKind::Comma).is_some()
					|| input.peek_matches(TokenKind::Ident).is_some()
				{
					let _comma = input.parse::<token::Comma>()?;
					Some((_self, Some(_comma)))
				} else {
					Some((_self, None))
				}
			}
			Err(_) => None
		};

		let args = input
			.try_parse::<Punctuated<Ident, token::Comma>>()
			.unwrap_or_default();

		Ok(match self_arg {
			Some((_self, _comma)) => {
				let args = _comma.map(|_c| (_c, args));
				Self::WithSelf { _self, args }
			}

			None => Self::WithoutSelf { args }
		})
	}
}
