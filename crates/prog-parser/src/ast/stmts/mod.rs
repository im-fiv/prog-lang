mod var_def;
mod var_assign;
mod do_block;
mod ret;
mod while_loop;
mod control_flow;
mod if_cond;
mod expr_assign;
mod class_def;

pub use var_def::VarDefine;
pub use var_assign::VarAssign;
pub use do_block::DoBlock;
pub use ret::Return;
pub use while_loop::WhileLoop;
pub use control_flow::{Break, Continue};
pub use if_cond::{If, ElseIf, Else};
pub use expr_assign::ExprAssign;
pub use class_def::ClassDef;

use anyhow::Result;
use prog_lexer::TokenKind;

use crate::{ast, ASTNode, Parse, ParseStream, Position, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct Program<'inp> {
	pub stmts: Vec<Statement<'inp>>
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement<'inp> {
	VarDefine(VarDefine<'inp>),
	VarAssign(VarAssign<'inp>),
	DoBlock(DoBlock<'inp>),
	Return(Return<'inp>),
	Call(ast::Call<'inp>),
	WhileLoop(WhileLoop<'inp>),
	Break(Break<'inp>),
	Continue(Continue<'inp>),
	If(If<'inp>),
	ExprAssign(ExprAssign<'inp>),
	ClassDef(ClassDef<'inp>)
}

impl ASTNode for Program<'_> {
	fn span(&self) -> Span {
		assert!(
			!self.stmts.is_empty(),
			"Could not get program's span as it is empty"
		);

		let first = self.stmts.first().unwrap().span();

		let start = first.position().start();
		let end = self.stmts.last().unwrap().end();

		let source = first.source();
		let position = Position::new(start, end);

		Span::new(source, position)
	}
}

impl<'inp> Parse<'inp> for Program<'inp> {
	fn parse(input: &ParseStream<'inp>) -> Result<Self> {
		let mut stmts = vec![];

		while let Some(token) = input.peek() {
			if token.kind() == TokenKind::Eof {
				break;
			}

			let stmt = input.parse::<Statement>()?;
			stmts.push(stmt);
		}

		Ok(Self { stmts })
	}
}

impl ASTNode for Statement<'_> {
	fn span(&self) -> Span {
		match self {
			Self::VarDefine(s) => s as &dyn ASTNode,
			Self::VarAssign(s) => s as &dyn ASTNode,
			Self::DoBlock(s) => s as &dyn ASTNode,
			Self::Return(s) => s as &dyn ASTNode,
			Self::Call(s) => s as &dyn ASTNode,
			Self::WhileLoop(s) => s as &dyn ASTNode,
			Self::Break(s) => s as &dyn ASTNode,
			Self::Continue(s) => s as &dyn ASTNode,
			Self::If(s) => s as &dyn ASTNode,
			Self::ExprAssign(s) => s as &dyn ASTNode,
			Self::ClassDef(s) => s as &dyn ASTNode
		}
		.span()
	}
}

impl<'inp> Parse<'inp> for Statement<'inp> {
	fn parse(input: &ParseStream<'inp>) -> Result<Self> {
		if input.peek_matches(TokenKind::Def).is_some() {
			return input
				.parse::<VarDefine>()
				.map(Self::VarDefine);
		}

		input.parse::<ast::Call>().map(Self::Call)
	}
}
