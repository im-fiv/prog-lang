mod ast;
mod errors;

use ast::*;
use errors::error;

use std::iter::Peekable;
use anyhow::Result;
use pest::iterators::{Pair, Pairs};
use pest::Parser;

#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
struct PestParser;

fn pair_into_string(pair: Pair<'_, Rule>) -> String {
	String::from(pair.as_span().as_str())
}

fn transform_ast(pairs: Pairs<'_, Rule>) -> Program {
	for pair in pairs {
		match pair.as_rule() {
			Rule::COMMENT |
			Rule::WHITESPACE |
			Rule::line_comment |
			Rule::block_comment => (),

			Rule::program => return Program { statements: parse_statements(pair.into_inner()) },

			rule => error!("expected '{:?}', got '{:?}'", pair.as_span(), Rule::program, rule)
		}
	}

	unreachable!("AST does not have a Program rule")
}

fn parse_statements(pairs: Pairs<'_, Rule>) -> Vec<statements::Statement> {
	let mut statements = vec![];

	for pair in pairs {
		if pair.as_rule() == Rule::EOI {
			continue;
		}

		let statement = match pair.as_rule() {
			Rule::var_define_stmt => parse_var_define_stmt(pair),
			Rule::var_assign_stmt => parse_var_assign_stmt(pair),

			rule => error!("statement of type '{:?}' is not yet implemented", pair.as_span(), rule)
		};

		statements.push(statement);
	}
	
	statements
}

fn parse_var_define_stmt(pair: Pair<'_, Rule>) -> statements::Statement {
	let mut pairs = pair.clone().into_inner();

	let name = pairs
		.next()
		.unwrap_or_else(|| error!("pair of type '{:?}' is missing in '{:?}'", pair.as_span(), Rule::identifier, pair.as_rule()));

	let value = pairs.next();

	if value.is_none() {
		return statements::Statement::VariableDefine(
			statements::VariableDefine {
				name: pair_into_string(name),
				value: None
			}
		);
	}

	let value = parse_expression(value.unwrap());

	statements::Statement::VariableDefine(
		statements::VariableDefine {
			name: pair_into_string(name),
			value: Some(value)
		}
	)
}

fn parse_var_assign_stmt(pair: Pair<'_, Rule>) -> statements::Statement {
	let mut pairs = pair.clone().into_inner();

	let name = pairs
		.next()
		.unwrap_or_else(|| error!("pair of type '{:?}' is missing in '{:?}'", pair.as_span(), Rule::identifier, pair.as_rule()));

	let value = pairs
		.next()
		.unwrap_or_else(|| error!("pair of type '{:?}' is missing in '{:?}'", pair.as_span(), Rule::expression, pair.as_rule()));

	let value = parse_expression(value);

	statements::Statement::VariableAssign(
		statements::VariableAssign {
			name: pair_into_string(name),
			value
		}
	)
}

fn is_term(pair: Pair<'_, Rule>) -> bool {
	matches!(
		pair.as_rule(),
		Rule::literal | Rule::identifier
	)
}

fn parse_expression(pair: Pair<'_, Rule>) -> expressions::Expression {
	let mut pairs = pair
		.clone()
		.into_inner()
		.peekable();

	if pairs.len() < 1 || is_term(pair.clone()) {
		return expressions::Expression::Term(parse_term(pair));
	}

	parse_expression_with_precedence(&mut pairs, 0)
}

fn get_operator_precedence(pair: &Pair<Rule>) -> u8 {
	expressions::operators::BinaryOperator::try_from(
		pair.as_str().to_owned()
	).unwrap().get_precedence()
}

fn parse_expression_with_precedence(pairs: &mut Peekable<Pairs<Rule>>, precedence: u8) -> expressions::Expression {
	todo!()
}

fn parse_unary_expression(pair: Pair<Rule>) -> expressions::Unary {
	let mut pairs = pair.into_inner();

	// let operator = parse_unary_operator(&pairs.next().expect("Failed to parse unary expression: no operator is present"));
	// let operand = parse_expression(pairs.next().expect("Failed to parse unary expression: no operand is present"));

	let operator = pairs
		.next()
		.unwrap_or_else(|| error!("pair of type '{:?}' is missing in '{:?}'", pair.as_span(), Rule::unary_operator, pair.as_rule()));

	expressions::Unary {
		operator,
		operand: Box::new(operand)
	}
}

fn parse_term(pair: Pair<'_, Rule>) -> expressions::Term {
	match pair.as_rule() {
		Rule::unary_expression => parse_unary_expression(pair),
		Rule::binary_expression | Rule::expression => parse_expression(pair),

		Rule::number_literal |
		Rule::string_literal |
		Rule::boolean_literal => parse_literal(pair),

		Rule::identifier => parse_identifier(pair),

		rule => error!("invalid expression rule '{:?}'", pair.as_span(), rule)
	}
}

fn parse_literal(pair: Pair<Rule>) -> expressions::Literal {
	match pair.as_rule() {
		Rule::number_literal => parse_number_literal(pair),
		Rule::string_literal => parse_string_literal(pair),
		Rule::boolean_literal => parse_boolean_literal(pair),

		rule => error!("expected number or string literal, got '{:?}'", pair.as_span(), rule)
	}
}

fn parse_identifier(pair: Pair<Rule>) -> expressions::Term {
	if pair.as_rule() != Rule::identifier {
		error!("expected '{:?}', got '{:?}'", pair.as_span(), Rule::identifier, pair.as_rule());
	}

	expressions::Term::Identifier(pair_into_string(pair))
}

fn parse_number_literal(pair: Pair<Rule>) -> expressions::Literal {
	let string = pair.as_str().to_owned();

	match string.parse::<f64>() {
		Ok(num) => expressions::Literal::Number(num),
		Err(_) => error!("failed to parse number literal '{}'", pair.as_span(), string)
	}
}

fn parse_string_literal(pair: Pair<Rule>) -> expressions::Literal {
	let literal = pair.as_str().to_owned();
	let clean_literal = literal.trim_start_matches(&['\'', '\"'][..]).trim_end_matches(&['\'', '\"'][..]);

	expressions::Literal::String(clean_literal.to_owned())
}

fn parse_boolean_literal(pair: Pair<Rule>) -> expressions::Literal {
	let literal = pair.as_str();

	match literal {
		"true" => expressions::Literal::Boolean(true),
		"false" => expressions::Literal::Boolean(false),

		_ => error!("expected 'true' or 'false', got '{}'", pair.as_span(), literal)
	}
}

pub fn parse(source: &str) -> Result<Program> {
	let token_tree = PestParser::parse(Rule::program, &source)?;
	let ast = transform_ast(token_tree);

	Ok(ast)
}