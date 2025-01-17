pub trait Token<'src> {
	fn tk(&self) -> prog_lexer::TokenKind;
	fn sp(&self) -> crate::Span<'src>;
}

macro_rules! def_token {
	($vis:vis $name:ident) => {
		#[derive(Debug, Clone, Copy, PartialEq, Hash)]
		#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
		$vis struct $name<'src> {
			span: ::prog_utils::pretty_errors::Span<'src>
		}

		impl<'src> $name<'src> {
			pub fn new(span: ::prog_utils::pretty_errors::Span<'src>) -> Self {
				Self { span }
			}
		}

		impl<'src> $crate::Token<'src> for $name<'src> {
			fn tk(&self) -> ::prog_lexer::TokenKind {
				::prog_lexer::TokenKind::$name
			}

			fn sp(&self) -> $crate::Span<'src> {
				self.span
			}
		}

		impl<'src> $crate::ASTNode<'src> for $name<'src> {
			fn span<'a>(&'a self) -> ::prog_utils::pretty_errors::Span<'src> {
				self.span
			}
		}

		impl<'src> $crate::Parse<'src> for $name<'src> {
			fn parse(input: &$crate::ParseStream<'src>) -> $crate::ParseResult<'src, Self> {
				let token = input.expect(::prog_lexer::TokenKind::$name)?;

				Ok(Self {
					span: token.span()
				})
			}
		}

		impl<'src> TryFrom<::prog_lexer::Token<'src>> for $name<'src> {
			type Error = $crate::ParseError<'src>;

			fn try_from(token: ::prog_lexer::Token<'src>) -> Result<Self, Self::Error> {
				let token = &token as &dyn $crate::Token;

				let token_kind = token.tk();
				let self_kind = ::prog_lexer::TokenKind::$name;

				if token_kind != self_kind {
					Err($crate::ParseError::new(
						token.sp(),
						$crate::ParseErrorKind::Internal($crate::error::Internal(
							format!("token of type `{token_kind}` cannot be converted to that of `{self_kind}`")
						))
					))
				} else {
					Ok(Self {
						span: token.sp()
					})
				}
			}
		}

		impl<'src> From<$name<'src>> for ::prog_lexer::Token<'src> {
			fn from(value: $name<'src>) -> Self {
				Self::new(
					::prog_lexer::TokenKind::$name,
					value.span
				)
			}
		}

		impl ::std::fmt::Display for $name<'_> {
			fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
				write!(f, "{}", self.tk())
			}
		}
	};
}

impl<'src> Token<'src> for prog_lexer::Token<'src> {
	fn tk(&self) -> prog_lexer::TokenKind { self.kind() }

	fn sp(&self) -> crate::Span<'src> { self.span() }
}

def_token!(pub True);
def_token!(pub False);
def_token!(pub Def);
def_token!(pub Func);
def_token!(pub Do);
def_token!(pub End);
def_token!(pub Return);
def_token!(pub While);
def_token!(pub Break);
def_token!(pub Continue);
def_token!(pub If);
def_token!(pub Then);
def_token!(pub ElseIf);
def_token!(pub Else);
def_token!(pub None);
def_token!(pub And);
def_token!(pub Or);
def_token!(pub Not);
def_token!(pub Class);
def_token!(pub Extern);

def_token!(pub Ident);
// Comments are ignored
// def_token!(pub Comment);
def_token!(pub Number);
def_token!(pub String);

def_token!(pub Plus);
def_token!(pub Minus);
def_token!(pub Asterisk);
def_token!(pub Slash);
def_token!(pub Sign);
def_token!(pub Eq);
def_token!(pub EqEq);
def_token!(pub Neq);
def_token!(pub Arrow);
def_token!(pub FatArrow);
def_token!(pub Dot);
def_token!(pub Comma);

def_token!(pub Lt);
def_token!(pub Gt);
def_token!(pub Lte);
def_token!(pub Gte);

def_token!(pub LeftParen);
def_token!(pub RightParen);
def_token!(pub LeftBrace);
def_token!(pub RightBrace);
def_token!(pub LeftBracket);
def_token!(pub RightBracket);

def_token!(pub Eof);
