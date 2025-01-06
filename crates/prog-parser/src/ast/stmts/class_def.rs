use anyhow::Result;

use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDef<'inp> {
	pub _class: token::Class<'inp>,
	pub name: token::Ident<'inp>,
	pub fields: Vec<VarDefine<'inp>>,
	pub _end: token::End<'inp>
}

impl ASTNode for ClassDef<'_> {
	fn span(&self) -> Span {
		let start = self._class.start();
		let end = self._end.end();

		let source = self._class.source();
		let position = Position::new(start, end);

		Span::new(source, position)
	}
}

impl<'inp> Parse<'inp> for ClassDef<'inp> {
	fn parse(input: &ParseStream<'inp>) -> Result<Self> {
		let _class = input.parse::<token::Class>()?;
		let name = input.parse::<token::Ident>()?;
		let mut fields = vec![];

		while let Ok(f) = input.try_parse::<VarDefine>() {
			fields.push(f);
		}

		let _end = input.parse::<token::End>()?;

		Ok(Self {
			_class,
			name,
			fields,
			_end
		})
	}
}
