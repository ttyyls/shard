use std::fmt::Display;

mod token;
pub use token::{Token, TokenKind};

use crate::report::{LogHandler, Report, ReportKind};
use crate::span::Span;

pub struct Lexer<'source> {
	contents: &'source str,
	index:    usize,
	span:     Span,

	handler:  LogHandler,
	tokens:   Vec<Token<'source>>,
}

impl<'source> Lexer<'source> {
	#[inline]
	fn report(&self, report: Report) {
		self.handler.log(report);
	}

	fn current(&self) -> Option<&'source str> {
		self.contents.get(self.index..=self.index)
	}

	fn peek(&self) -> Option<&'source str> {
		self.contents.get(self.index + 1..=self.index + 1)
	}

	fn slice_source(&self, index: usize, len: usize) -> &'source str {
		&self.contents[index..index + len]
	}

	fn push_token(&mut self, kind: TokenKind, span: Span, text: &'source str) {
		self.tokens.push(Token { kind, span, text });
	}

	fn push_token_simple(&mut self, kind: TokenKind, length: usize) {
		let (index, span) = (self.index, self.span);

		(0..length).for_each(|_| self.advance());
		self.push_token(kind, span.len(length), self.slice_source(index, length));
	}

	fn advance(&mut self) {
		self.index += 1;
		self.span.offset += 1;

		if Some("\n") == self.current() {
			self.span.line_number += 1;
			self.span.offset = 0;
		}
	}

	pub fn tokenize(filename: &'static str, contents: &'source str, handler: LogHandler) -> Vec<Token<'source>> {
		let mut lex = Self {
			contents,
			handler,
			index:  0,
			span:   Span::new(filename, 1, 0, 0),
			tokens: Vec::new(),
		};

		'outer: while let Some(current) = lex.current() {
			let (index, span) = (lex.index, lex.span);

			let (token, len) = match current {
				"\n" => {
					while Some("\n") == lex.current() {
						lex.advance();
					}

					if lex.tokens.last().is_some_and(|t| t.kind != TokenKind::NewLine) {
						lex.push_token(TokenKind::NewLine, lex.span, "");
					}
					continue;
				},

				c if c.chars().any(char::is_whitespace) => {
					lex.advance();
					continue;
				},

				"/" => match lex.peek() {
					Some("/") => {
						while Some("\n") != lex.current() {
							lex.advance();
						}
						continue;
					},
					Some("*") => {
						let mut depth = 0;
						loop {
							match lex.current() {
								Some("/") if Some("*") == lex.peek() => {
									lex.advance();
									lex.advance();
									depth += 1;
								},
								Some("*") if Some("/") == lex.peek() => {
									lex.advance();
									lex.advance();
									depth -= 1;
								},
								None => break,
								_ => lex.advance(),
							}

							if depth == 0 {
								break;
							}
						}

						if depth > 0 {
							lex.report(
								ReportKind::UnterminatedMultilineComment
									.title(format!("{depth} comments never terminated"))
									.span(lex.span),
							);
						}

						continue;
					},
					_ => (TokenKind::Slash, 1),
				},

				c if c.chars().any(|c| c.is_ascii_alphabetic()) => {
					while let Some(c) = lex.current() {
						if c.chars().any(|c| c.is_ascii_alphanumeric() || c == '_') {
							lex.advance();
							continue;
						}
						break;
					}

					let ident = lex.slice_source(index, lex.index - index);
					let kind = match ident {
						"ret" => TokenKind::KeywordRet,
						"struct" => TokenKind::KeywordStruct,
						"enum" => TokenKind::KeywordEnum,
						"destr" => TokenKind::KeywordDestr,
						"type" => TokenKind::KeywordType,
						"op" => TokenKind::KeywordOp,
						"cast" => TokenKind::KeywordCast,
						"extern" => TokenKind::KeywordExtern,
						_ => TokenKind::Identifier,
					};

					lex.push_token(kind, span.len(lex.index - index), ident);
					continue;
				},

				"\"" => {
					lex.advance();
					let span = lex.span;
					while let Some(c) = lex.current() {
						match c {
							"\"" => break,
							"\\" => {
								lex.advance();
								if lex.current() == Some("\"") {
									lex.advance();
								}
							},
							"\n" => {
								lex.report(
									ReportKind::UnterminatedStringLiteral
										.untitled()
										.span(span.offset(span.offset - 2).len(lex.index - index)),
								);
								continue 'outer;
							},
							_ => lex.advance(),
						}
					}
					let span = span.len(lex.index - (index + 1));
					lex.push_token(
						TokenKind::StringLiteral,
						span,
						lex.slice_source(index + 1, lex.index - (index + 1)),
					);

					lex.advance();

					continue;
				},

				"`" => {
					lex.advance();
					let start = lex.index;
					while let Some(c) = lex.current() {
						match c {
							"`" => {
								if lex.index == start {
									lex.report(
										ReportKind::EmptyCharLiteral
											.untitled()
											.span(span.len(2).offset(span.offset - 1)),
									);
									lex.advance();
									continue 'outer;
								}

								lex.advance();
								break;
							},

							"\\" => {
								lex.advance();
								if lex.current() == Some("`") && lex.peek() != Some("`") {
									lex.advance();
									lex.report(
										ReportKind::UnterminatedCharLiteral
											.untitled()
											.span(span.len(lex.index - index))
											.help("Remove the escape character"),
									);
									continue 'outer;
								}

								lex.advance();
							},

							"\n" => {
								lex.report(
									ReportKind::UnterminatedCharLiteral
										.untitled()
										.span(span.len(lex.index - index).offset(span.offset - 1)),
								);
								continue 'outer;
							},

							_ => lex.advance(),
						}
					}

					lex.push_token(
						TokenKind::CharLiteral,
						span.len(lex.index - index),
						lex.slice_source(index, lex.index - index),
					);

					continue;
				},

				"0" if lex.peek().filter(|c| "box".contains(c)).is_some() => {
					let (kind, base) = match lex.peek() {
						Some("b") => (TokenKind::BinaryIntLiteral, 2),
						Some("o") => (TokenKind::OctalIntLiteral, 8),
						Some("x") => (TokenKind::HexadecimalIntLiteral, 16),
						_ => unreachable!(),
					};

					lex.advance();
					lex.advance();
					if !lex.lex_integer(base) {
						continue;
					}

					lex.push_token(
						kind,
						lex.span.len(lex.index - index),
						lex.slice_source(index, lex.index - index),
					);

					continue;
				},

				c if c.chars().any(|c| c.is_ascii_digit()) => {
					if !lex.lex_integer(10) {
						continue;
					}

					if lex.current() == Some(".") {
						lex.advance();
						if !lex.lex_integer(10) {
							continue;
						}

						if lex.current() == Some(".") {
							lex.report(
								ReportKind::SyntaxError
									.title("Invalid Float Literal")
									.span(lex.span.len(1)),
							);
							lex.advance();
							continue;
						}

						lex.push_token(
							TokenKind::FloatLiteral,
							span.len(lex.index - index),
							lex.slice_source(index, lex.index - index),
						);

						continue;
					}

					lex.push_token(
						TokenKind::DecimalIntLiteral,
						span.len(lex.index - index),
						lex.slice_source(index, lex.index - index),
					);
					continue;
				},

				"." => (TokenKind::Dot, 1),
				"'" => (TokenKind::Apostrophe, 1),
				"~" => match lex.peek() {
					Some("=") => (TokenKind::NotEquals, 2),
					_ => (TokenKind::Tilde, 1),
				},
				"!" => (TokenKind::Bang, 1),
				"@" => (TokenKind::At, 1),
				"#" => (TokenKind::Pound, 1),
				"$" => (TokenKind::Dollar, 1),
				"%" => (TokenKind::Percent, 1),
				"^" => match lex.peek() {
					Some("^") => (TokenKind::CaretCaret, 2),
					_ => (TokenKind::Caret, 1),
				},
				"&" => match lex.peek() {
					Some("&") => (TokenKind::AmpersandAmpersand, 2),
					_ => (TokenKind::Ampersand, 1),
				},
				"*" => (TokenKind::Star, 1),
				"(" => (TokenKind::LParen, 1),
				")" => (TokenKind::RParen, 1),
				"-" => match lex.peek() {
					Some(">") => (TokenKind::ArrowRight, 2),
					Some("-") => (TokenKind::MinusMinus, 2),
					_ => (TokenKind::Minus, 1),
				},
				"_" => (TokenKind::Underscore, 1),
				"+" => match lex.peek() {
					Some("+") => (TokenKind::PlusPlus, 2),
					_ => (TokenKind::Plus, 1),
				},
				"[" => (TokenKind::LBracket, 1),
				"]" => (TokenKind::RBracket, 1),
				"{" => (TokenKind::LBrace, 1),
				"}" => (TokenKind::RBrace, 1),
				"|" => match lex.peek() {
					Some("|") => (TokenKind::PipePipe, 2),
					_ => (TokenKind::Pipe, 1),
				},
				";" => (TokenKind::Semicolon, 1),
				":" => (TokenKind::Colon, 1),
				"," => (TokenKind::Comma, 1),
				"=" => match lex.peek() {
					// Some("=") => (TokenKind::EqualsEquals, 2),
					Some(">") => (TokenKind::FatArrowRight, 2),
					_ => (TokenKind::Equals, 1),
				},
				"<" => match lex.peek() {
					Some("=") => (TokenKind::LessThanEquals, 2),
					Some("-") => (TokenKind::ArrowLeft, 2),
					Some("<") => (TokenKind::ShiftLeft, 2),
					_ => (TokenKind::LessThan, 1),
				},
				">" => match lex.peek() {
					Some("=") => (TokenKind::GreaterThanEquals, 2),
					Some(">") => (TokenKind::ShiftRight, 2),
					_ => (TokenKind::GreaterThan, 1),
				},
				"?" => (TokenKind::Question, 1),

				c => {
					lex.report(ReportKind::UnexpectedCharacter.title(c).span(lex.span));
					lex.advance();
					continue;
				},
			};
			lex.push_token_simple(token, len);
		}

		lex.tokens
	}

	fn lex_integer(&mut self, base: usize) -> bool {
		const CHARS: [char; 16] =
		['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'];

		while let Some(c) = self.current() {
			match (base, c.to_ascii_lowercase().chars().next().unwrap()) {
				(2, c) if CHARS[..1].contains(&c) => self.advance(),
				(8, c) if CHARS[..7].contains(&c) => self.advance(),
				(10, c) if CHARS[..9].contains(&c) => self.advance(),
				(16, c) if CHARS.contains(&c) => self.advance(),
				(_, '_') => self.advance(),

				(_, c) if c.is_ascii_alphanumeric() => {
					self.report(
						ReportKind::SyntaxError
							.title("Invalid Integer Literal")
							.span(self.span.len(1).offset(self.span.offset - 1))
							.label(format!("{c:?} not valid for base{base} Integer Literal")),
					);
					return false;
				},

				_ => break,
			}
		}
		true
	}
}
