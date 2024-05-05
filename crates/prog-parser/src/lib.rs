pub mod ast;
mod errors_legacy;
mod errors;
mod utils;

use ast::*;
use errors_legacy::error;
use utils::*;

use std::iter::Peekable;
use anyhow::{Result, bail};
use pest::iterators::{Pair, Pairs};
use pest::Span;

pub use errors::{ParseError, ParseErrorKind};

#[inline]
fn span_to_pos(span: Span) -> Position {
	span.start()..span.end()
}

#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
struct PestParser;

pub struct Parser<'inp> {
	source: &'inp str,
	file: &'inp str
}

impl<'inp> Parser<'inp> {
	pub fn new(source: &'inp str, file: &'inp str) -> Self {
		Self { source, file }
	}
	
	pub fn parse(&self) -> Result<Program> {
		use pest::Parser;
	
		let tt = PestParser::parse(Rule::program, self.source);

		if let Err(e) = tt {
			bail!(self.convert_error(e));
		}

		let ast = self.transform_ast(tt.unwrap());
	
		Ok(ast)
	}

	fn convert_error(&self, pest_error: pest::error::Error<Rule>) -> ParseError {
		let positives = match pest_error.variant {
			pest::error::ErrorVariant::ParsingError { positives, negatives: _ } => positives,
			_ => panic!("Only ErrorVariant::ParsingError is suitable for conversion")
		};
	
		let position = match pest_error.location {
			pest::error::InputLocation::Pos(p) => p..p,
			pest::error::InputLocation::Span(s) => (s.0)..(s.1)
		};
	
		ParseError::new(
			self.source.to_owned(),
			self.file.to_owned(),
			position.clone(),
			ParseErrorKind::ExpectedRules(
				errors::ExpectedRules(positives)
			)
		)
	}

	fn transform_ast(&self, pairs: Pairs<'_, Rule>) -> Program {
		for pair in pairs {
			match pair.as_rule() {
				Rule::COMMENT |
				Rule::WHITESPACE |
				Rule::line_comment |
				Rule::block_comment => (),
	
				Rule::program => return Program {
					statements: self.parse_statements(pair.into_inner())
				},
	
				rule => error!("expected '{:?}', got '{:?}'", pair.as_span(), Rule::program, rule)
			}
		}
	
		unreachable!("AST does not have a Program rule")
	}
	
	fn parse_statements(&self, pairs: Pairs<'_, Rule>) -> Vec<Statement> {
		let mut statements = vec![];
	
		for pair in pairs {
			if pair.as_rule() == Rule::EOI {
				continue;
			}
	
			if pair.as_rule() == Rule::statements {
				return self.parse_statements(pair.into_inner());
			}
	
			let statement = match pair.as_rule() {
				Rule::var_define_stmt => self.parse_var_define_stmt(pair),
				Rule::var_assign_stmt => self.parse_var_assign_stmt(pair),
				Rule::do_block => self.parse_do_block(pair),
				Rule::return_stmt => self.parse_return_stmt(pair),
				Rule::call => self.parse_function_call(pair).into(),
				Rule::while_stmt => self.parse_while_stmt(pair),
				
				Rule::break_stmt => Statement::Break(span_to_pos(pair.as_span())),
				Rule::continue_stmt => Statement::Continue(span_to_pos(pair.as_span())),
	
				Rule::if_stmt => self.parse_if_stmt(pair),
	
				Rule::expr_assign_stmt => self.parse_expr_assign_stmt(pair),
	
				rule => error!("statement of type '{:?}' is not yet implemented", pair.as_span(), rule)
			};
	
			statements.push(statement);
		}
		
		statements
	}
	
	fn parse_var_define_stmt(&self, pair: Pair<'_, Rule>) -> Statement {
		assert_rule!(pair == var_define_stmt in pair);
		let mut pairs = pair.clone().into_inner();
	
		let name_pair = get_pair_safe!(from pairs expect identifier in pair);
		let name_position = span_to_pos(name_pair.as_span());
		let name = pair_into_string(
			&name_pair
		);
	
		let value = pairs.next();
	
		if value.is_none() {
			return Statement::VariableDefine {
				name: (name, name_position),
				value: None,
				position: span_to_pos(pair.as_span())
			};
		}
	
		let value = self.parse_expression(value.unwrap());
	
		Statement::VariableDefine {
			name: (name, name_position),
			value: Some(value),
			position: span_to_pos(pair.as_span())
		}
	}
	
	fn parse_var_assign_stmt(&self, pair: Pair<'_, Rule>) -> Statement {
		assert_rule!(pair == var_assign_stmt in pair);

		let position = span_to_pos(pair.as_span());
		let mut pairs = pair.clone().into_inner();
	
		let name_pair = get_pair_safe!(from pairs expect identifier in pair);
		let name_position = span_to_pos(name_pair.as_span());
		let name = pair_into_string(
			&name_pair
		);
	
		let value = self.parse_expression(
			get_pair_safe!(from pairs expect expression in pair)
		);
	
		Statement::VariableAssign {
			name: (name, name_position),
			value,
			position
		}
	}
	
	fn parse_do_block(&self, pair: Pair<'_, Rule>) -> Statement {
		assert_rule!(pair == do_block in pair);
	
		let position = span_to_pos(pair.as_span());
		let statements = self.parse_statements(pair.into_inner());

		Statement::DoBlock(statements, position)
	}
	
	fn parse_return_stmt(&self, pair: Pair<'_, Rule>) -> Statement {
		assert_rule!(pair == return_stmt in pair);
	
		let position = span_to_pos(pair.as_span());
		let value = pair
			.into_inner()
			.next();
	
		if value.is_none() {
			return Statement::Return(None, position);
		}
	
		let parsed_expression = self.parse_expression(value.unwrap());

		if let Expression::Empty(_) = parsed_expression {
			return Statement::Return(None, position);
		}
	
		Statement::Return(
			Some(parsed_expression),
			position
		)
	}
	
	fn parse_list(&self, pair: Pair<'_, Rule>) -> expressions::List {
		assert_rule!(pair == list in pair);

		let position = span_to_pos(pair.as_span());
		let pairs = pair.clone().into_inner();
		let mut expressions = vec![];
	
		for expression_pair in pairs {
			assert_rule!(expression_pair == expression in pair);
	
			let parsed_pair = self.parse_expression(expression_pair);
			expressions.push(parsed_pair);
		}
	
		expressions::List(expressions, position)
	}
	
	fn parse_object(&self, pair: Pair<'_, Rule>) -> expressions::Object {
		assert_rule!(pair == object in pair);

		let position = span_to_pos(pair.as_span());
		let pairs = pair.clone().into_inner();
		let mut entries = vec![];
	
		for entry_pair in pairs {
			assert_rule!(entry_pair == object_entry in pair);
	
			entries.push(
				self.parse_object_entry(entry_pair)
			);
		}
	
		expressions::Object(entries, position)
	}
	
	fn parse_object_entry(&self, pair: Pair<'_, Rule>) -> expressions::ObjectEntry {
		assert_rule!(pair == object_entry in pair);

		let position = span_to_pos(pair.as_span());
		let mut pairs = pair.clone().into_inner();
	
		let name = pair_into_string(
			&get_pair_safe!(from pairs expect identifier in pair)
		);
	
		let value = self.parse_expression(
			get_pair_safe!(from pairs expect expression in pair)
		);
	
		expressions::ObjectEntry { name, value, position }
	}
	
	fn parse_function_call(&self, pair: Pair<'_, Rule>) -> expressions::Call {
		assert_rule!(pair == call in pair);
	
		let position = span_to_pos(pair.as_span());
		let mut pairs = pair.clone().into_inner();
		let mut arguments = vec![];
	
		let next_pair = get_pair_safe!(from pairs expect call_body_empty | call_body_nonempty in pair);
		let arguments_pos = span_to_pos(next_pair.as_span());
	
		if next_pair.as_rule() == Rule::call_body_nonempty {
			arguments = self.parse_function_call_args(next_pair);
		}
	
		let function = self.parse_expression(
			get_pair_safe!(from pairs expect expression in pair)
		);
	
		expressions::Call {
			arguments: (arguments, arguments_pos),
			function: Box::new(function),
			position
		}
	}
	
	fn parse_function_call_args(&self, pair: Pair<'_, Rule>) -> Vec<Expression> {
		assert_rule!(pair == call_body_nonempty in pair);
		
		let pairs = pair.clone().into_inner();
		let mut arguments = vec![];
	
		for arg_pair in pairs {
			assert_rule!(arg_pair == expression in pair);
			arguments.push(self.parse_expression(arg_pair));
		}
	
		arguments
	}
	
	fn parse_while_stmt(&self, pair: Pair<'_, Rule>) -> Statement {
		assert_rule!(pair == while_stmt in pair);

		let position = span_to_pos(pair.as_span());
		let mut pairs = pair.clone().into_inner();
	
		let condition = self.parse_expression(
			get_pair_safe!(from pairs expect expression in pair)
		);
	
		let statements = self.parse_statements(
			get_pair_safe!(from pairs expect do_block in pair).into_inner()
		);
	
		Statement::WhileLoop { condition, statements, position }
	}
	
	fn parse_if_stmt(&self, pair: Pair<'_, Rule>) -> Statement {
		assert_rule!(pair == if_stmt in pair);

		let position = span_to_pos(pair.as_span());
		let mut pairs = pair.clone().into_inner();
	
		let condition = self.parse_expression(
			get_pair_safe!(from pairs expect expression in pair)
		);
	
		let statements = self.parse_statements(
			get_pair_safe!(from pairs expect statements in pair).into_inner()
		);
	
		let mut elseif_branches: Vec<ConditionBranch> = vec![];
		let mut else_branch: Option<ConditionBranch> = None;
	
		for pair in pairs {
			assert_rule!(pair == if_elseif | if_else in pair);
			
			if pair.as_rule() == Rule::if_elseif {
				elseif_branches.push(self.parse_elseif_branch(pair));
			} else {
				else_branch = Some(self.parse_else_branch(pair));
			}
		}
	
		Statement::If { condition, statements, elseif_branches, else_branch, position }
	}
	
	fn parse_elseif_branch(&self, pair: Pair<'_, Rule>) -> ConditionBranch {
		assert_rule!(pair == if_elseif in pair);

		let position = span_to_pos(pair.as_span());
		let mut pairs = pair.clone().into_inner();
	
		let condition = self.parse_expression(
			get_pair_safe!(from pairs expect expression in pair)
		);
	
		let statements = self.parse_statements(
			get_pair_safe!(from pairs expect statements in pair).into_inner()
		);
	
		ConditionBranch { condition, statements, position }
	}
	
	fn parse_else_branch(&self, pair: Pair<'_, Rule>) -> ConditionBranch {
		assert_rule!(pair == if_else in pair);

		let position = span_to_pos(pair.as_span());
		let mut pairs = pair.clone().into_inner();
	
		let statements = self.parse_statements(
			get_pair_safe!(from pairs expect statements in pair).into_inner()
		);
	
		ConditionBranch {
			condition: Expression::Empty(None),
			statements,
			position
		}
	}
	
	fn parse_expr_assign_stmt(&self, pair: Pair<'_, Rule>) -> Statement {
		assert_rule!(pair == expr_assign_stmt in pair);

		let position = span_to_pos(pair.as_span());
		let mut pairs = pair.clone().into_inner();
	
		let expression = self.parse_expression(
			get_pair_safe!(from pairs expect expression in pair)
		);
	
		let value = self.parse_expression(
			get_pair_safe!(from pairs expect expression in pair)
		);
	
		Statement::ExpressionAssign { expression, value, position }
	}
	
	fn parse_expression_with_precedence(&self, pairs: &mut Peekable<Pairs<Rule>>, precedence: u8) -> Expression {
		if pairs.len() < 1 {
			unreachable!("Failed to parse expression: pairs are empty");
		}
	
		let left_pair = pairs
			.next()
			.unwrap();
	
		let mut position = span_to_pos(left_pair.as_span());
		let mut left = self.parse_term(left_pair);
	
		while let Some(pair) = pairs.peek() {
			if pair.as_rule() != Rule::binary_operator {
				break;
			}
	
			let operator_position = span_to_pos(pair.as_span());
			let operator = get_bin_operator_from_pair(pair);
			let operator_precedence = operator.get_precedence();
			
			if operator_precedence < precedence {
				break;
			}
			
			// Consume the operator
			pairs.next();
	
			let right = self.parse_expression_with_precedence(pairs, operator_precedence + 1);
			position = position.start..right.position().end;
	
			left = Expression::Binary(
				expressions::Binary {
					lhs: left.clone(),
					operator: (operator, operator_position),
					rhs: right.into(),
					position: position.clone()
				}
			).into()
		}
	
		left.into()
	}
	
	fn parse_term(&self, pair: Pair<'_, Rule>) -> expressions::Term {
		// In accordance with `src/crates/prog-parser/src/grammar.pest:44`
		assert_rule!(pair == unary_expression | binary_expression | term in pair);

		match pair.as_rule() {
			Rule::unary_expression => return self.parse_unary_expression(pair).into(),
			Rule::binary_expression | Rule::expression => return self.parse_expression(pair).into(),

			_ => ()
		}

		let inner_pair = pair
			.into_inner()
			.next()
			.unwrap_or_else(|| panic!("inner pairs of term are empty"));

		match inner_pair.as_rule() {
			Rule::object => self.parse_object(inner_pair).into(),
			Rule::list => self.parse_list(inner_pair).into(),
	
			Rule::call => self.parse_function_call(inner_pair).into(),
			Rule::function => self.parse_function(inner_pair).into(),
	
			Rule::unary_expression => self.parse_unary_expression(inner_pair).into(),
			Rule::binary_expression | Rule::expression => self.parse_expression(inner_pair).into(),
	
			Rule::number_literal |
			Rule::string_literal |
			Rule::boolean_literal => self.parse_literal(inner_pair).into(),
	
			Rule::identifier => self.parse_identifier(inner_pair),
	
			rule => error!("unsupported term rule '{:?}'", inner_pair.as_span(), rule)
		}
	}
	
	fn parse_function(&self, pair: Pair<'_, Rule>) -> expressions::Function {
		assert_rule!(pair == function in pair);

		let position = span_to_pos(pair.as_span());
		let mut pairs = pair.clone().into_inner();
		let mut arguments = vec![];
	
		let mut next_pair = get_pair_safe!(from pairs expect function_def_args | do_block in pair);
	
		if next_pair.as_rule() == Rule::function_def_args {
			arguments = self.parse_function_def_args(next_pair);
			next_pair = get_pair_safe!(from pairs expect do_block in pair);
		}
	
		// We don't call `parse_do_block` because it's a pain in the ass to extract the statements from there
		let statements = self.parse_statements(next_pair.into_inner());
	
		expressions::Function { arguments, statements, position }
	}
	
	fn parse_function_def_args(&self, pair: Pair<'_, Rule>) -> Vec<(String, Position)> {
		assert_rule!(pair == function_def_args in pair);
		
		let pairs = pair.clone().into_inner();
		let mut arguments = vec![];
	
		for arg_pair in pairs {
			assert_rule!(arg_pair == identifier in pair);
			let position = span_to_pos(pair.as_span());

			arguments.push((
				pair_into_string(&arg_pair),
				position
			));
		}
	
		arguments
	}
	
	fn parse_expression(&self, pair: Pair<'_, Rule>) -> Expression {
		let mut pairs = pair
			.clone()
			.into_inner()
			.peekable();
	
		if pairs.len() < 1 || pair.as_rule() == Rule::term {
			return Expression::Term(self.parse_term(pair));
		}
	
		self.parse_expression_with_precedence(&mut pairs, 0)
	}
	
	fn parse_unary_expression(&self, pair: Pair<Rule>) -> expressions::Unary {
		assert_rule!(pair == unary_expression in pair);

		let position = span_to_pos(pair.as_span());
		let mut pairs = pair.clone().into_inner();
	
		let operator_pair = get_pair_safe!(from pairs expect unary_operator in pair);
		let operator_position = span_to_pos(operator_pair.as_span());
		let operator = expressions::operators::UnaryOperator::try_from(
			pair_into_string(&operator_pair)
		).unwrap();

		let operand = self.parse_term(
			get_pair_safe!(from pairs expect term in pair)
		);
	
		expressions::Unary {
			operator: (operator, operator_position),
			operand,
			position
		}
	}
	
	fn parse_literal(&self, pair: Pair<Rule>) -> expressions::Literal {
		assert_rule!(pair == number_literal | string_literal | boolean_literal in pair);
	
		match pair.as_rule() {
			Rule::number_literal => self.parse_number_literal(pair),
			Rule::string_literal => self.parse_string_literal(pair),
			Rule::boolean_literal => self.parse_boolean_literal(pair),
	
			_ => unreachable!()
		}
	}
	
	fn parse_identifier(&self, pair: Pair<Rule>) -> expressions::Term {
		assert_rule!(pair == identifier in pair);

		let position = span_to_pos(pair.as_span());
		let as_str = pair_into_string(&pair);
	
		if as_str == "void" {
			return Expression::Empty(Some(position)).into();
		}
	
		expressions::Term::Identifier(as_str, position)
	}
	
	fn parse_number_literal(&self, pair: Pair<Rule>) -> expressions::Literal {
		assert_rule!(pair == number_literal in pair);

		let position = span_to_pos(pair.as_span());
		let string = pair.as_str().to_owned();
	
		match string.parse::<f64>() {
			Ok(num) => expressions::Literal::Number(num, position),
			Err(_) => error!("failed to parse number literal '{}'", pair.as_span(), string)
		}
	}
	
	fn parse_string_literal(&self, pair: Pair<Rule>) -> expressions::Literal {
		assert_rule!(pair == string_literal in pair);

		let position = span_to_pos(pair.as_span());
		let literal = pair.as_str().to_owned();
		let clean_literal = literal.trim_start_matches(&['\'', '\"'][..]).trim_end_matches(&['\'', '\"'][..]);
	
		expressions::Literal::String(
			clean_literal.to_owned(),
			position
		)
	}
	
	fn parse_boolean_literal(&self, pair: Pair<Rule>) -> expressions::Literal {
		assert_rule!(pair == boolean_literal in pair);

		let position = span_to_pos(pair.as_span());
		let literal = pair.as_str();
	
		match literal {
			"true" => expressions::Literal::Boolean(true, position),
			"false" => expressions::Literal::Boolean(false, position),
	
			_ => error!("expected 'true' or 'false', got '{}'", pair.as_span(), literal)
		}
	}
}