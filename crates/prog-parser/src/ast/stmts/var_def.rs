use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseResult, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum VarDefine<'src> {
	WithValue {
		_def: token::Def<'src>,
		name: Ident<'src>,
		_eq: token::Eq<'src>,
		value: Expr<'src>
	},

	NoValue {
		_def: token::Def<'src>,
		name: Ident<'src>
	}
}

impl<'src> VarDefine<'src> {
	pub fn _def(&self) -> token::Def<'src> {
		match self {
			Self::WithValue { _def, .. } => *_def,
			Self::NoValue { _def, .. } => *_def
		}
	}

	pub fn name(&self) -> Ident<'src> {
		match self {
			Self::WithValue { name, .. } => *name,
			Self::NoValue { name, .. } => *name
		}
	}

	pub fn _eq(&self) -> Option<token::Eq<'src>> {
		match self {
			Self::WithValue { _eq, .. } => Some(*_eq),
			Self::NoValue { .. } => None
		}
	}

	pub fn value(&self) -> Option<Expr<'src>> {
		match self {
			Self::WithValue { value, .. } => Some(value.clone()),
			Self::NoValue { .. } => None
		}
	}
}

impl ASTNode for VarDefine<'_> {
	fn span(&self) -> Span {
		match self {
			Self::WithValue {
				_def,
				name: _,
				_eq,
				value
			} => {
				let start = _def.start();
				let end = value.end();

				let source = _def.source();
				let file = _def.file();
				let position = Position::new(start, end);

				Span::new(source, file, position)
			}

			Self::NoValue { _def, name } => {
				let start = _def.start();
				let end = name.end();

				let source = _def.source();
				let file = _def.file();
				let position = Position::new(start, end);

				Span::new(source, file, position)
			}
		}
	}
}

impl<'src> Parse<'src> for VarDefine<'src> {
	fn parse(input: &ParseStream<'src>) -> ParseResult<Self> {
		let _def = input.parse::<token::Def>()?;
		let name = input.parse::<Ident>()?;

		let Ok(_eq) = input.try_parse::<token::Eq>() else {
			return Ok(Self::NoValue { _def, name });
		};

		let value = input.parse::<Expr>()?;

		Ok(Self::WithValue {
			_def,
			name,
			_eq,
			value
		})
	}
}
