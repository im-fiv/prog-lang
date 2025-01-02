macro_rules! def_token {
	($vis:vis $name:ident) => {
		#[derive(Debug, Clone, Copy, PartialEq, Hash)]
		$vis struct $name<'inp> {
			span: prog_utils::pretty_errors::Span<'inp>
		}

		impl<'inp> From<prog_lexer::Token<'inp>> for $name<'inp> {
			fn from(value: prog_lexer::Token<'inp>) -> Self {
				Self {
					span: value.span()
				}
			}
		}

		impl<'inp> Token<'inp> for $name<'inp> {
			fn kind(&self) -> prog_lexer::TokenKind {
				prog_lexer::TokenKind::$name
			}

			fn span(&self) -> prog_utils::pretty_errors::Span {
				self.span
			}
		}

		impl<'inp> crate::Parse<'inp> for $name<'inp> {
			fn parse(input: &'inp crate::ParseStream) -> anyhow::Result<Self> {
				let token = input.expect(prog_lexer::TokenKind::$name)?;

				Ok(Self {
					span: token.span()
				})
			}
		}
	};
}

pub trait Token<'inp>: From<prog_lexer::Token<'inp>> {
	fn kind(&self) -> prog_lexer::TokenKind;

	fn span(&self) -> prog_utils::pretty_errors::Span;

	fn value(&'inp self) -> &'inp str { self.span().value() }
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
def_token!(pub None);
def_token!(pub And);
def_token!(pub Or);
def_token!(pub Not);
def_token!(pub Class);
def_token!(pub Extern);

def_token!(pub Identifier);
// Comments are ignored
// def_token!(pub Comment);
def_token!(pub Number);
def_token!(pub String);

def_token!(pub Plus);
def_token!(pub Minus);
def_token!(pub Asterisk);
def_token!(pub Slash);
def_token!(pub Eq);
def_token!(pub EqEq);
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