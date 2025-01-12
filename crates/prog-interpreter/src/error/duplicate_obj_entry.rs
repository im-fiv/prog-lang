use ariadne::{ColorGenerator, Label};
use prog_parser::ASTNode;
use prog_utils::pretty_errors::{AriadneCompatible, Span};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct DuplicateObjEntry<'ast> {
	/// Span of the originally defined entry.
	pub def: prog_parser::ast::ObjEntry<'ast>
}

impl AriadneCompatible for DuplicateObjEntry<'_> {
	fn message(&self) -> String { String::from("duplicate object entry") }

	fn labels(self, span: Span) -> Vec<ariadne::Label<Span>> {
		let mut colors = ColorGenerator::new();

		let color_redef = colors.next();
		let color_def = colors.next();

		vec![
			Label::new(span)
				.with_message(format!("entry `{}`", self.def.name))
				.with_color(color_redef),
			
			Label::new(self.def.span())
				.with_message("has already been defined here")
				.with_color(color_def)
		]
	}
}
