use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub struct Identifier(pub String);

impl From<String> for Identifier {
	fn from(value: String) -> Self {
		Self(value)
	}
}

impl Display for Identifier {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}