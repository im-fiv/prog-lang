use crate::ast::*;
use crate::{token, ASTNode, Parse, ParseResult, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ClassDef<'src> {
	pub _class: token::Class<'src>,
	pub name: Ident<'src>,
	pub fields: Vec<VarDefine<'src>>,
	pub _end: token::End<'src>
}

impl<'src> ASTNode<'src> for ClassDef<'src> {
	fn span<'a>(&'a self) -> Span<'src> {
		let start = self._class.start();
		let end = self._end.end();

		let source = self._class.source();
		let file = self._class.file();
		let position = Position::new(start, end);

		Span::new(source, file, position)
	}
}

impl<'src> Parse<'src> for ClassDef<'src> {
	fn parse(input: &ParseStream<'src, '_>) -> ParseResult<'src, Self> {
		let _class = input.parse::<token::Class>()?;
		let name = input.parse::<Ident>()?;
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
