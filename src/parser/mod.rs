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

fn parse_statements(pairs: Pairs<'_, Rule>) -> Vec<Statement> {
	let mut statements = vec![];

	for pair in pairs {
		if pair.as_rule() == Rule::EOI {
			continue;
		}

		let statement = match pair.as_rule() {
			Rule::var_define_stmt => parse_var_define_stmt(pair),
			Rule::var_assign_stmt => parse_var_assign_stmt(pair),
			Rule::do_block => parse_do_block(pair),
			Rule::return_stmt => parse_return_stmt(pair),
			Rule::call => parse_function_call(pair).into(),

			rule => error!("statement of type '{:?}' is not yet implemented", pair.as_span(), rule)
		};

		statements.push(statement);
	}
	
	statements
}

fn parse_var_define_stmt(pair: Pair<'_, Rule>) -> Statement {
	let mut pairs = pair.clone().into_inner();

	let name = pairs
		.next()
		.unwrap_or_else(|| error!("pair of type '{:?}' is missing in '{:?}'", pair.as_span(), Rule::identifier, pair.as_rule()));

	let value = pairs.next();

	if value.is_none() {
		return Statement::VariableDefine {
			name: pair_into_string(name),
			value: None
		};
	}

	let value = parse_expression(value.unwrap());

	Statement::VariableDefine {
		name: pair_into_string(name),
		value: Some(value)
	}
}

fn parse_var_assign_stmt(pair: Pair<'_, Rule>) -> Statement {
	let mut pairs = pair.clone().into_inner();

	let name = pairs
		.next()
		.unwrap_or_else(|| error!("pair of type '{:?}' is missing in '{:?}'", pair.as_span(), Rule::identifier, pair.as_rule()));

	let value = pairs
		.next()
		.unwrap_or_else(|| error!("pair of type '{:?}' is missing in '{:?}'", pair.as_span(), Rule::expression, pair.as_rule()));

	let value = parse_expression(value);

	Statement::VariableAssign {
		name: pair_into_string(name),
		value
	}
}

fn parse_do_block(pair: Pair<'_, Rule>) -> Statement {
	let statements = parse_statements(pair.into_inner());
	Statement::DoBlock(statements)
}

fn parse_return_stmt(pair: Pair<'_, Rule>) -> Statement {
	let value = pair
		.into_inner()
		.next();

	if value.is_none() {
		return Statement::Return(None);
	}

	Statement::Return(Some(parse_expression(value.unwrap())))
}

fn parse_function_call(pair: Pair<'_, Rule>) -> expressions::Call {
	let mut pairs = pair.clone().into_inner();
	let mut arguments: Vec<expressions::Expression> = vec![];

	let next_pair = pairs
		.next()
		.unwrap_or_else(|| error!("pair of type '{:?}' or '{:?}' is missing from '{:?}'", pair.as_span(), Rule::call_body_empty, Rule::call_body_nonempty, pair.as_rule()));

	if !matches!(next_pair.as_rule(), Rule::call_body_empty | Rule::call_body_nonempty) {
		error!("invalid pair of type '{:?}' in '{:?}' (expected '{:?}' or '{:?}')", pair.as_span(), next_pair.as_rule(), pair.as_rule(), Rule::call_body_empty, Rule::call_body_nonempty)
	}

	if next_pair.as_rule() == Rule::call_body_nonempty {
		arguments = parse_function_call_args(next_pair);
	}

	let function_pair = pairs
		.next()
		.unwrap_or_else(|| error!("pair of type '{:?}' is missing from '{:?}'", pair.as_span(), Rule::expression, pair.as_rule()));

	if function_pair.as_rule() != Rule::expression {
		error!("invalid pair of type '{:?}' in '{:?}' (expected '{:?}')", function_pair.as_span(), function_pair.as_rule(), pair.as_rule(), Rule::expression);
	}

	let function = parse_expression(function_pair);
	expressions::Call { arguments, function: Box::new(function) }
}

fn parse_function_call_args(pair: Pair<'_, Rule>) -> Vec<expressions::Expression> {
	let pairs = pair.clone().into_inner();
	let mut arguments = vec![];

	for arg_pair in pairs {
		if arg_pair.as_rule() != Rule::expression {
			error!("invalid pair of type '{:?}' in '{:?}' (expected '{:?}')", arg_pair.as_span(), arg_pair.as_rule(), pair.as_rule(), Rule::expression);
		}

		arguments.push(parse_expression(arg_pair));
	}

	arguments
}

fn is_term(pair: Pair<'_, Rule>) -> bool {
	matches!(
		pair.as_rule(),
		Rule::literal | Rule::identifier
	)
}

fn get_bin_op_from_pair(pair: &Pair<'_, Rule>) -> expressions::operators::BinaryOperator {
	expressions::operators::BinaryOperator::try_from(
		pair.as_str().to_owned()
	).unwrap()
}

fn parse_expression_with_precedence(pairs: &mut Peekable<Pairs<Rule>>, precedence: u8) -> expressions::Expression {
	if pairs.len() < 1 {
		unreachable!("Failed to parse expression: pairs are empty");
	}

	let left_pair = pairs
		.next()
		.unwrap();

	let mut left = parse_term(left_pair);

	while let Some(pair) = pairs.peek() {
		if pair.as_rule() == Rule::binary_operator {
			let operator = get_bin_op_from_pair(pair);
			let operator_precedence = operator.get_precedence();
			
			if operator_precedence < precedence {
				break;
			}
			
			// Consume the operator
			pairs.next();

			let right = parse_expression_with_precedence(pairs, operator_precedence + 1);

			left = expressions::Expression::Binary(
				expressions::Binary {
					lhs: left.clone(),
					operator,
					rhs: right.into()
				}
			).into()
		} else {
			break;
		}
	}

	left.try_into().unwrap()
}

fn parse_term(pair: Pair<'_, Rule>) -> expressions::Term {
	match pair.as_rule() {
		Rule::call => parse_function_call(pair).into(),
		Rule::function => parse_function(pair).into(),

		Rule::unary_expression => parse_unary_expression(pair).into(),
		Rule::binary_expression | Rule::expression => parse_expression(pair).into(),

		Rule::number_literal |
		Rule::string_literal |
		Rule::boolean_literal => parse_literal(pair).into(),

		Rule::identifier => parse_identifier(pair),

		rule => error!("unsupported expression rule '{:?}'", pair.as_span(), rule)
	}
}

fn parse_function(pair: Pair<'_, Rule>) -> expressions::Function {
	let mut pairs = pair.clone().into_inner();

	let mut next_pair = pairs
		.next()
		.unwrap_or_else(|| error!("pair of type '{:?}' or '{:?}' is missing in '{:?}'", pair.as_span(), Rule::function_def_args, Rule::do_block, pair.as_rule()));

	let mut arguments = vec![];

	if next_pair.as_rule() == Rule::function_def_args {
		arguments = parse_function_def_args(next_pair);

		next_pair = pairs
			.next()
			.unwrap_or_else(|| error!("pair of type '{:?}' is missing in '{:?}'", pair.as_span(), Rule::do_block, pair.as_rule()));
	}

	if next_pair.as_rule() != Rule::do_block {
		error!("invalid pair of type '{:?}' in '{:?}' (expected '{:?}')", next_pair.as_span(), next_pair.as_rule(), pair.as_rule(), Rule::do_block)
	}

	// we don't call `parse_do_block` because it's a pain in the ass to extract the statements from there
	let statements = parse_statements(next_pair.into_inner());

	expressions::Function { arguments, statements }
}

fn parse_function_def_args(pair: Pair<'_, Rule>) -> Vec<String> {
	let pairs = pair.clone().into_inner();
	let mut arguments = vec![];

	for arg_pair in pairs {
		if arg_pair.as_rule() != Rule::identifier {
			error!("invalid pair of type '{:?}' in '{:?}' (expected '{:?}')", pair.as_span(), arg_pair.as_rule(), pair.as_rule(), Rule::identifier)
		}

		arguments.push(pair_into_string(arg_pair));
	}

	arguments
}

fn parse_expression(pair: Pair<'_, Rule>) -> expressions::Expression {
	let mut pairs = pair
		.clone()
		.into_inner()
		.peekable();

	if pairs.len() < 1 || is_term(pair.clone()) {
		// TODO: perhaps rewrite that?
		return expressions::Expression::Term(parse_term(pair));
	}

	parse_expression_with_precedence(&mut pairs, 0)
}

fn parse_unary_expression(pair: Pair<Rule>) -> expressions::Unary {
	let mut pairs = pair.clone().into_inner();

	let operator_pair = pairs
		.next()
		.unwrap_or_else(|| error!("pair of type '{:?}' is missing in '{:?}'", pair.as_span(), Rule::unary_operator, pair.as_rule()));

	let operator = expressions::operators::UnaryOperator::try_from(
		pair_into_string(operator_pair)
	).unwrap();

	let operand_pair = pairs
		.next()
		.unwrap_or_else(|| error!("pair of type '{:?}' is missing in '{:?}'", pair.as_span(), Rule::term, pair.as_rule()));

	let operand = parse_term(operand_pair);

	expressions::Unary {
		operator,
		operand
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
	let tt = PestParser::parse(Rule::program, &source)?;
	let ast = transform_ast(tt);

	Ok(ast)
}