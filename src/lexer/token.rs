use std::fmt::Formatter;

use colored::Colorize;

use crate::span::Span;

#[derive(Debug, PartialOrd, PartialEq, Clone, Copy)]
pub enum TokenKind {
	NewLine,
	Identifier,

	KeywordRet,
	KeywordStruct,
	KeywordEnum,
	KeywordDestr,
	KeywordType,
	KeywordOp,
	KeywordCast,
	KeywordExtern,

	FloatLiteral,

	BinaryIntLiteral,
	OctalIntLiteral,
	DecimalIntLiteral,
	HexadecimalIntLiteral,

	StringLiteral,
	CharLiteral,

	Tilde,
	Bang,
	At,
	Pound,
	Dollar,
	Percent,
	Caret,
	CaretCaret,
	Ampersand,
	AmpersandAmpersand,
	Star,
	LParen,
	RParen,
	Minus,
	Underscore,
	Equals,
	Plus,
	LBracket,
	RBracket,
	LBrace,
	RBrace,
	Pipe,
	PipePipe,
	Semicolon,
	Colon,
	Comma,
	Dot,
	Slash,
	Question,
	ArrowLeft,
	ArrowRight,
	FatArrowRight,
	GreaterThan,
	GreaterThanEquals,
	LessThan,
	LessThanEquals,
	MinusMinus,
	NotEquals,
	PlusPlus,
	ShiftLeft,
	ShiftRight,
	Apostrophe,
}

#[derive(Debug, Copy, Clone)]
pub struct Token<'source> {
	pub kind: TokenKind, // this gets padded to 8 urghh 7 bytes lost why cant we have nice things
	pub span: Span, // FIXME: this already has len, ideally we wouldn't store that in the span
	pub text: &'source str, // make this a ptr?
}

impl std::fmt::Display for Token<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "Token({:?}, {}", self.kind, format!("{:?}", self.span).bright_black())?;
		if !self.text.is_empty() { write!(f, ", {}", format!("{:?}", self.text).green())?; }
		write!(f, ")")
	}
}

impl TokenKind {
	pub fn matching(self) -> Self {
		match self {
			Self::LBrace   => Self::RBrace,
			Self::RBrace   => Self::LBrace,
			Self::LBracket => Self::RBracket,
			Self::RBracket => Self::LBracket,
			Self::LParen   => Self::RParen,
			Self::RParen   => Self::LParen,
			a => a,
		}
	}
}
