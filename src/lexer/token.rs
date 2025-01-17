use std::fmt::Formatter;

use colored::Colorize;

use crate::span::Span;

#[derive(Debug, PartialOrd, PartialEq, Clone, Copy)]
pub enum TokenKind {
	Identifier,

	KWFn,
	KWExport,
	KWRet,
	KWStruct,
	KWEnum,
	KWImpl,
	KWType,
	KWExtern,

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

	EOF,
}

#[derive(Debug, Copy, Clone)]
pub struct Token<'source> {
	pub kind: TokenKind,
	pub span: Span,
	pub text: &'source str, // TODO: remove text, just slice from source with span
}

impl std::fmt::Display for Token<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "Token({:?}, {}", self.kind, 
			format!("{}-{}", self.span.start, self.span.end).bright_black())?;
		if !self.text.is_empty() { write!(f, ", {}", format!("{:?}", self.text).green())?; }
		write!(f, ")")
	}
}
