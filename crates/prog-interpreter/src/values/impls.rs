use super::*;

//* From<T> *//

impl From<ast::expressions::Literal> for RuntimeValue {
	fn from(value: ast::expressions::Literal) -> Self {
		use ast::expressions::Literal;

		match value {
			Literal::Boolean(value, _) => Self::Boolean(value),
			Literal::String(value, _) => Self::String(value),
			Literal::Number(value, _) => Self::Number(value)
		}
	}
}

impl From<String> for Identifier {
	fn from(value: String) -> Self {
		Self(value)
	}
}

//* Display *//

impl Display for RuntimeValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let fmt_list = |f: &mut std::fmt::Formatter<'_>, value: &Vec<RuntimeValue>| {
			let formatted = value
				.iter()
				.map(|entry| entry.to_string())
				.collect::<Vec<String>>()
				.join(", ");
			
			write!(f, "[{formatted}]")
		};

		let fmt_object = |f: &mut std::fmt::Formatter<'_>, value: &HashMap<String, RuntimeValue>| {
			let formatted = value
				.iter()
				.map(|(name, value)| format!("{name} = {value}"))
				.collect::<Vec<String>>()
				.join(", ");
			
			write!(f, "{{ {formatted} }}")
		};
		
		match self {
			Self::Boolean(value) => write!(f, "{}", if value.to_owned() { "true" } else { "false" }),
			Self::String(value) => write!(f, "{value}"),
			Self::Number(value) => write!(f, "{value}"),
			Self::List(value) => fmt_list(f, value),
			Self::Object(value) => fmt_object(f, value),

			Self::Function(value) => write!(f, "{value}"),
			Self::IntrinsicFunction(value) => write!(f, "{value}"),

			Self::Identifier(value) => write!(f, "{}", value.0),
			Self::Empty => write!(f, "")
		}
	}
}

impl Display for RuntimeFunction {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let arguments_str = self.arguments.join(", ");
		let formatted = format!("Function({arguments_str})");
		
		write!(f, "{formatted}")
	}
}

impl Display for IntrinsicFunction {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "func({:?})", self.pointer)
	}
}