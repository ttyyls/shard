use crate::lexer::{Token, TokenKind};
use crate::LogHandler;
use crate::report::{ReportKind, Report, Result};

pub enum AST<'src> {
	Module(&'src str, Vec<AST<'src>>),
	Func {
		name: &'src str, 
		linkage: bool, // TODO: move separate struc; true = export
		args: Vec<(&'src str, AST<'src>)>,
		ret:  Box<AST<'src>>,
		body: Vec<AST<'src>>,
	},
	Type(&'src str),
}

pub struct Parser<'src> {
	tokens:  Vec<Token<'src>>,
	index:   usize,
}

impl<'src> Parser<'src> {
	#[inline]
	fn current(&self) -> Result<Token<'src>> {
		match self.tokens.get(self.index).copied() {
			Some(token) => Ok(token),
			None => ReportKind::UnexpectedEOF
				.untitled()
				.span(self.peek(-1).unwrap().span)
				.as_err(),
		}
	}

	#[inline]
	fn is_end(&self) -> bool {
		self.index >= self.tokens.len()
	}

	#[inline]
	#[allow(clippy::cast_possible_wrap)]
	fn peek(&self, index: isize) -> Option<&Token<'src>> {
		let index = self.index as isize + index;
		assert!(index >= 0, "peek() out of bounds");
		self.tokens.get(unsafe { std::mem::transmute::<isize, usize>(index) })
	}

	#[inline]
	fn advance(&mut self) {
		self.index += 1;
		// assert!(self.index < self.tokens.len(), "advance() out of bounds");
	}

	pub fn parse(tokens: Vec<Token<'src>>, filename: &'src str, handler: &LogHandler) -> AST<'src> {
		let mut ast = AST::Module(filename, Vec::new());

		if tokens.is_empty() { return ast; }

		let mut parser = Self {
			tokens,
			index: 0,
		};

		while !parser.is_end() {
			let AST::Module(_, ref mut globals) = ast
				else { unreachable!() };

			match parser.parse_global() {
				Ok(global)  => globals.push(global),
				Err(report) => handler.log(report),
			}

			parser.advance();
		}

		ast
	}

	fn parse_global(&mut self) -> Result<AST<'src>> {
		let token = self.current()?;

		match token.kind {
			TokenKind::KWFn => self.parse_func(),
			TokenKind::KWExport => { // FIXME: come up with a better way to do this
				self.advance();
				if self.is_end() { panic!("Unexpected end of file") }

				match self.parse_global()? {
					AST::Func { name, args, ret, body, .. } 
						=> Ok(AST::Func { name, args, ret, body, linkage: true }),
					// TODO: const/static
					_ => panic!("Expected function"),
				}
			},
			_ => ReportKind::UnexpectedToken
				.untitled().span(token.span).as_err(),
		}
	}

	fn parse_func(&mut self) -> Result<AST<'src>> {
		self.advance();

		let token = self.current()?;
		let name = match token.kind {
			TokenKind::Identifier => token.text,
			_ => return ReportKind::UnexpectedToken
				.title("Expected identifier")
				.span(token.span).as_err(),
		};

		self.advance();

		// TODO: generic parsing
		
		if self.current()?.kind != TokenKind::LParen {
			return ReportKind::UnexpectedToken
				.title("Expected '('")
				.span(self.current()?.span).as_err();
		}

		let mut args = Vec::new();
		loop {
			self.advance();
			let token = self.current()?;
			match token.kind {
				TokenKind::RParen => break,
				TokenKind::Identifier => {
					let name = token.text;
					self.advance();

					let token = self.current()?;
					if token.kind != TokenKind::Colon {
						return ReportKind::UnexpectedToken
							.title("Expected ':'")
							.span(token.span).as_err();
					}

					self.advance();

					args.push((name, self.parse_type()?));
				},
				_ => return ReportKind::UnexpectedToken
					.title("Expected identifier")
					.span(token.span).as_err(),
			}
		}

		self.advance();
		let ret = Box::new(self.parse_type()?);

		self.advance();
		let body = self.parse_block()?;

		Ok(AST::Func { name, args, ret, body, linkage: false })
	}

	fn parse_block(&mut self) -> Result<Vec<AST<'src>>> {
		todo!()
	}

	fn parse_type(&mut self) -> Result<AST<'src>> {
		todo!()
	}
}
