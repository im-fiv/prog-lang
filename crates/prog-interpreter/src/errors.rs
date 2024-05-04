use ariadne::{ColorGenerator, Label, Fmt};
use prog_utils::pretty_errors::{Position, Span, PrettyError, AriadneCompatible};

pub type InterpretError = PrettyError<InterpretErrorKind>;

#[derive(Debug, Clone)]
pub enum InterpretErrorKind {
	UnsupportedStatement(String, Position)
}

impl AriadneCompatible for InterpretErrorKind {
	fn message(&self) -> String {
		String::from("unsupported statement")
	}

	fn labels<'a>(&'a self, file: &'a str) -> Vec<Label<Span>> {
		match self {
			Self::UnsupportedStatement(statement, position) => {
				let mut colors = ColorGenerator::new();
				let color = colors.next();

				let position = position.to_owned();

				vec![
					Label::new((file, position))
						.with_message(
							format!(
								"`{}` is unsupported",
								statement.fg(color)
							)
						)
						.with_color(color)
				]
			}
		}
	}
}