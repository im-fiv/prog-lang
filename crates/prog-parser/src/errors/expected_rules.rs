use ariadne::{ColorGenerator, Label};
use prog_utils::pretty_errors::{AriadneCompatible, Position, Span};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExpectedRules(pub Vec<crate::Rule>);

impl AriadneCompatible for ExpectedRules {
	fn message(&self) -> String { String::from("unexpected rule") }

	fn labels(self, file: &str, position: Position) -> Vec<Label<Span>> {
		let mut colors = ColorGenerator::new();
		let color = colors.next();

		let rules = self
			.0
			.into_iter()
			.map(|rule| format!("`{:?}`", rule).replace('_', " "))
			.collect::<Vec<_>>()
			.join(", ");

		vec![Label::new(Span::new_unchecked(file, position))
			.with_message(format!("expected {}", rules))
			.with_color(color)]
	}
}

#[cfg(feature = "serde")]
impl serde::Serialize for crate::Rule {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(&format!("{:?}", self))
	}
}
