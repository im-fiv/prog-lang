mod var_def;
mod var_assign;
mod do_block;
mod ret;
mod while_loop;
mod control_flow;
mod if_cond;
mod expr_assign;
mod class_def;

pub use class_def::ClassDef;
pub use control_flow::{Break, Continue};
pub use do_block::DoBlock;
pub use expr_assign::ExprAssign;
pub use if_cond::{Else, ElseIf, If};
pub use ret::Return;
pub use var_assign::VarAssign;
pub use var_def::VarDefine;
pub use while_loop::WhileLoop;

use prog_lexer::TokenKind;

use crate::{
	ast, error, ASTNode, Parse, ParseError, ParseErrorKind, ParseResult, ParseStream, Span
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Stmt<'src> {
	VarDefine(VarDefine<'src>),
	VarAssign(VarAssign<'src>),
	DoBlock(DoBlock<'src>),
	Return(Return<'src>),
	Call(ast::Call<'src>),
	WhileLoop(WhileLoop<'src>),
	Break(Break<'src>),
	Continue(Continue<'src>),
	If(If<'src>),
	ExprAssign(ExprAssign<'src>),
	ClassDef(ClassDef<'src>)
}

impl ASTNode for Stmt<'_> {
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

impl<'src> Parse<'src> for Stmt<'src> {
	fn parse(input: &ParseStream<'src>) -> ParseResult<Self> {
		let token = input.expect_peek()?;
		let span = token.span();

		// `def ...`
		if input.peek_matches(TokenKind::Def).is_some() {
			return input.parse::<VarDefine>().map(Self::VarDefine);
		}

		// `do ...`
		if input.peek_matches(TokenKind::Do).is_some() {
			return input.parse::<DoBlock>().map(Self::DoBlock);
		}

		// `return ...`
		if input.peek_matches(TokenKind::Return).is_some() {
			return input.parse::<Return>().map(Self::Return);
		}

		// `break`
		if let Ok(stmt) = input.try_parse::<Break>() {
			return Ok(Self::Break(stmt));
		}

		// `continue ...`
		if let Ok(stmt) = input.try_parse::<Continue>() {
			return Ok(Self::Continue(stmt));
		}

		// `while ...`
		if input.peek_matches(TokenKind::While).is_some() {
			return input.parse::<WhileLoop>().map(Self::WhileLoop);
		}

		// `if ...`
		if input.peek_matches(TokenKind::If).is_some() {
			return input.parse::<If>().map(Self::If);
		}

		// `class ...`
		if input.peek_matches(TokenKind::Class).is_some() {
			return input.parse::<ClassDef>().map(Self::ClassDef);
		}

		if let Ok(stmt) = input.try_parse::<ExprAssign>() {
			return Ok(Self::ExprAssign(stmt));
		} else if let Ok(stmt) = input.try_parse::<ast::Call>() {
			return Ok(Self::Call(stmt));
		}

		Err(ParseError::with_span(
			span,
			ParseErrorKind::Internal(error::Internal(String::from("no statement matched")))
		))
	}
}
