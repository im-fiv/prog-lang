pub mod ast;
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

macro_rules! assert_rule {
	($var:ident == $rule:ident $(| $rest:ident)* in $main_pair:expr) => {
		if !matches!($var.as_rule(), Rule::$rule $(| Rule::$rest)*) {
			let expected_str = assert_rule!(format_expected $rule $($rest),*);

			error!(
				"invalid pair of type '{:?}' in '{:?}' (expected {})", $var.as_span(),
				$var.as_rule(),
				$main_pair.as_rule(),
				expected_str
			);
		}
	};

	(format_expected $rule:ident $($rest:ident),*) => {
		[
			format!("'{:?}'", Rule::$rule)
			$(, format!(", '{:?}'", Rule::$rest))*
		].concat()
	};
}

macro_rules! get_pair_safe {
	(from $pairs:ident expect $rule:ident $(| $rest:ident)* in $main_pair:expr) => {
		{
			let expected_str = assert_rule!(format_expected $rule $($rest),*);

			let next_pair = $pairs
				.next()
				.unwrap_or_else(|| error!(
					"pair of type {} is missing in '{:?}'",
					$main_pair.as_span(),
					expected_str,
					$main_pair.as_rule()
				));

			next_pair
		}
	};
}

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

		if pair.as_rule() == Rule::statements {
			return parse_statements(pair.into_inner());
		}

		let statement = match pair.as_rule() {
			Rule::var_define_stmt => parse_var_define_stmt(pair),
			Rule::var_assign_stmt => parse_var_assign_stmt(pair),
			Rule::do_block => parse_do_block(pair),
			Rule::return_stmt => parse_return_stmt(pair),
			Rule::call => parse_function_call(pair).into(),
			Rule::while_stmt => parse_while_stmt(pair),
			
			Rule::break_stmt => Statement::Break,
			Rule::continue_stmt => Statement::Continue,

			Rule::if_stmt => parse_if_stmt(pair),

			rule => error!("statement of type '{:?}' is not yet implemented", pair.as_span(), rule)
		};

		statements.push(statement);
	}
	
	statements
}

fn parse_var_define_stmt(pair: Pair<'_, Rule>) -> Statement {
	assert_rule!(pair == var_define_stmt in pair);
	let mut pairs = pair.clone().into_inner();

	let name = pair_into_string(
		get_pair_safe!(from pairs expect identifier in pair)
	);

	let value = pairs.next();

	if value.is_none() {
		return Statement::VariableDefine {
			name,
			value: None
		};
	}

	let value = parse_expression(value.unwrap());

	Statement::VariableDefine {
		name,
		value: Some(value)
	}
}

fn parse_var_assign_stmt(pair: Pair<'_, Rule>) -> Statement {
	assert_rule!(pair == var_assign_stmt in pair);
	let mut pairs = pair.clone().into_inner();

	let name = pair_into_string(
		get_pair_safe!(from pairs expect identifier in pair)
	);

	let value = parse_expression(
		get_pair_safe!(from pairs expect expression in pair)
	);

	Statement::VariableAssign {
		name,
		value
	}
}

fn parse_do_block(pair: Pair<'_, Rule>) -> Statement {
	assert_rule!(pair == do_block in pair);

	let statements = parse_statements(pair.into_inner());
	Statement::DoBlock(statements)
}

fn parse_return_stmt(pair: Pair<'_, Rule>) -> Statement {
	assert_rule!(pair == return_stmt in pair);

	let value = pair
		.into_inner()
		.next();

	if value.is_none() {
		return Statement::Return(None);
	}

	let parsed_expression = parse_expression(value.unwrap());
	
	if matches!(parsed_expression, Expression::Empty) {
		return Statement::Return(None);
	}

	Statement::Return(
		Some(parsed_expression)
	)
}

fn parse_list(pair: Pair<'_, Rule>) -> expressions::List {
	assert_rule!(pair == list in pair);

	let pairs = pair.clone().into_inner();
	let mut expressions = vec![];

	for expression_pair in pairs {
		assert_rule!(expression_pair == expression in pair);

		let parsed_pair = parse_expression(expression_pair);
		expressions.push(parsed_pair);
	}

	expressions::List(expressions)
}

fn parse_function_call(pair: Pair<'_, Rule>) -> expressions::Call {
	assert_rule!(pair == call in pair);

	let mut pairs = pair.clone().into_inner();
	let mut arguments: Vec<Expression> = vec![];

	let next_pair = get_pair_safe!(from pairs expect call_body_empty | call_body_nonempty in pair);

	if next_pair.as_rule() == Rule::call_body_nonempty {
		arguments = parse_function_call_args(next_pair);
	}

	let function = parse_expression(
		get_pair_safe!(from pairs expect expression in pair)
	);

	expressions::Call { arguments, function: Box::new(function) }
}

fn parse_function_call_args(pair: Pair<'_, Rule>) -> Vec<Expression> {
	assert_rule!(pair == call_body_nonempty in pair);
	let pairs = pair.clone().into_inner();
	let mut arguments = vec![];

	for arg_pair in pairs {
		assert_rule!(arg_pair == expression in pair);
		arguments.push(parse_expression(arg_pair));
	}

	arguments
}

fn parse_while_stmt(pair: Pair<'_, Rule>) -> Statement {
	assert_rule!(pair == while_stmt in pair);
	let mut pairs = pair.clone().into_inner();

	let condition = parse_expression(
		get_pair_safe!(from pairs expect expression in pair)
	);

	let statements = parse_statements(
		get_pair_safe!(from pairs expect do_block in pair).into_inner()
	);

	Statement::WhileLoop { condition, statements }
}

fn parse_if_stmt(pair: Pair<'_, Rule>) -> Statement {
	assert_rule!(pair == if_stmt in pair);
	let mut pairs = pair.clone().into_inner();

	let condition = parse_expression(
		get_pair_safe!(from pairs expect expression in pair)
	);

	let statements = parse_statements(
		get_pair_safe!(from pairs expect statements in pair).into_inner()
	);

	let mut elseif_branches: Vec<ConditionBranch> = vec![];
	let mut else_branch: Option<ConditionBranch> = None;

	for pair in pairs {
		assert_rule!(pair == if_elseif | if_else in pair);
		
		if pair.as_rule() == Rule::if_elseif {
			elseif_branches.push(parse_elseif_branch(pair));
		} else {
			else_branch = Some(parse_else_branch(pair));
		}
	}

	Statement::If { condition, statements, elseif_branches, else_branch }
}

fn parse_elseif_branch(pair: Pair<'_, Rule>) -> ConditionBranch {
	assert_rule!(pair == if_elseif in pair);
	let mut pairs = pair.clone().into_inner();

	let condition = parse_expression(
		get_pair_safe!(from pairs expect expression in pair)
	);

	let statements = parse_statements(
		get_pair_safe!(from pairs expect statements in pair).into_inner()
	);

	ConditionBranch { condition, statements }
}

fn parse_else_branch(pair: Pair<'_, Rule>) -> ConditionBranch {
	assert_rule!(pair == if_else in pair);
	let mut pairs = pair.clone().into_inner();

	let statements = parse_statements(
		get_pair_safe!(from pairs expect statements in pair).into_inner()
	);

	ConditionBranch { condition: Expression::Empty, statements }
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

fn parse_expression_with_precedence(pairs: &mut Peekable<Pairs<Rule>>, precedence: u8) -> Expression {
	if pairs.len() < 1 {
		unreachable!("Failed to parse expression: pairs are empty");
	}

	let left_pair = pairs
		.next()
		.unwrap();

	let mut left = parse_term(left_pair);

	while let Some(pair) = pairs.peek() {
		if pair.as_rule() != Rule::binary_operator {
			break;
		}

		let operator = get_bin_op_from_pair(pair);
		let operator_precedence = operator.get_precedence();
		
		if operator_precedence < precedence {
			break;
		}
		
		// Consume the operator
		pairs.next();

		let right = parse_expression_with_precedence(pairs, operator_precedence + 1);

		left = Expression::Binary(
			expressions::Binary {
				lhs: left.clone(),
				operator,
				rhs: right.into()
			}
		).into()
	}

	left.into()
}

fn parse_term(pair: Pair<'_, Rule>) -> expressions::Term {
	match pair.as_rule() {
		Rule::list => parse_list(pair).into(),
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
	assert_rule!(pair == function in pair);
	let mut pairs = pair.clone().into_inner();
	let mut arguments = vec![];

	let mut next_pair = get_pair_safe!(from pairs expect function_def_args | do_block in pair);

	if next_pair.as_rule() == Rule::function_def_args {
		arguments = parse_function_def_args(next_pair);
		next_pair = get_pair_safe!(from pairs expect do_block in pair);
	}

	// We don't call `parse_do_block` because it's a pain in the ass to extract the statements from there
	let statements = parse_statements(next_pair.into_inner());

	expressions::Function { arguments, statements }
}

fn parse_function_def_args(pair: Pair<'_, Rule>) -> Vec<String> {
	assert_rule!(pair == function_def_args in pair);
	let pairs = pair.clone().into_inner();
	let mut arguments = vec![];

	for arg_pair in pairs {
		assert_rule!(arg_pair == identifier in pair);
		arguments.push(pair_into_string(arg_pair));
	}

	arguments
}

fn parse_expression(pair: Pair<'_, Rule>) -> Expression {
	let mut pairs = pair
		.clone()
		.into_inner()
		.peekable();

	if pairs.len() < 1 || is_term(pair.clone()) {
		// TODO: perhaps rewrite that?
		return Expression::Term(parse_term(pair));
	}

	parse_expression_with_precedence(&mut pairs, 0)
}

fn parse_unary_expression(pair: Pair<Rule>) -> expressions::Unary {
	assert_rule!(pair == unary_expression in pair);
	let mut pairs = pair.clone().into_inner();

	let operator = expressions::operators::UnaryOperator::try_from(
		pair_into_string(get_pair_safe!(from pairs expect unary_operator in pair))
	).unwrap();

	let operand = parse_term(
		get_pair_safe!(from pairs expect term in pair)
	);

	expressions::Unary {
		operator,
		operand
	}
}

fn parse_literal(pair: Pair<Rule>) -> expressions::Literal {
	assert_rule!(pair == number_literal | string_literal | boolean_literal in pair);

	match pair.as_rule() {
		Rule::number_literal => parse_number_literal(pair),
		Rule::string_literal => parse_string_literal(pair),
		Rule::boolean_literal => parse_boolean_literal(pair),

		_ => unreachable!()
	}
}

fn parse_identifier(pair: Pair<Rule>) -> expressions::Term {
	assert_rule!(pair == identifier in pair);
	let as_str = pair_into_string(pair);

	if as_str == "void" {
		return Expression::Empty.into();
	}

	expressions::Term::Identifier(as_str)
}

fn parse_number_literal(pair: Pair<Rule>) -> expressions::Literal {
	assert_rule!(pair == number_literal in pair);
	let string = pair.as_str().to_owned();

	match string.parse::<f64>() {
		Ok(num) => expressions::Literal::Number(num),
		Err(_) => error!("failed to parse number literal '{}'", pair.as_span(), string)
	}
}

fn parse_string_literal(pair: Pair<Rule>) -> expressions::Literal {
	assert_rule!(pair == string_literal in pair);
	let literal = pair.as_str().to_owned();
	let clean_literal = literal.trim_start_matches(&['\'', '\"'][..]).trim_end_matches(&['\'', '\"'][..]);

	expressions::Literal::String(clean_literal.to_owned())
}

fn parse_boolean_literal(pair: Pair<Rule>) -> expressions::Literal {
	assert_rule!(pair == boolean_literal in pair);
	let literal = pair.as_str();

	match literal {
		"true" => expressions::Literal::Boolean(true),
		"false" => expressions::Literal::Boolean(false),

		_ => error!("expected 'true' or 'false', got '{}'", pair.as_span(), literal)
	}
}

pub fn parse(source: &str) -> Result<Program> {
	let tt = PestParser::parse(Rule::program, source)?;
	let ast = transform_ast(tt);

	Ok(ast)
}