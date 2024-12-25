use crate::lexer::{Token, TokenKind};

enum AST {
	Add(Vec<AST>),
	UInt(usize),
}

impl AST {
	fn from_lisp(tokens: Vec<Token>) -> Self {
	}
}

struct Parser {
	tokens: Vec<String>,
}
