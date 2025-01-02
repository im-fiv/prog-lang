use anyhow::Result;
use token::Token;

use crate::{token, Parse, ParseStream};

#[derive(Debug)]
pub struct Number(f64);

impl<'inp> Parse<'inp> for Number {
	fn parse(input: &'inp ParseStream<'inp>) -> Result<Self> {
		let token = input.parse::<token::Number>()?;
		let value = token.value().parse::<f64>().unwrap();

		Ok(Self(value))
	}
}

#[derive(Debug)]
pub struct VariableDefinition<'inp> {
	_def: token::Def<'inp>,
	name: token::Identifier<'inp>,
	_eq: token::Eq<'inp>,
	value: Number
}

impl<'inp> Parse<'inp> for VariableDefinition<'inp> {
	fn parse(input: &'inp ParseStream<'inp>) -> Result<Self> {
		let _def = input.parse::<token::Def>()?;
		let name = input.parse::<token::Identifier>()?;
		let _eq = input.parse::<token::Eq>()?;
		let value = input.parse::<Number>()?;

		Ok(Self {
			_def,
			name,
			_eq,
			value
		})
	}
}
