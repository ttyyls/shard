use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicUsize, Ordering};

use colored::{Color, Colorize};
pub use progress::LogHandler;

use crate::util::CACHE;
use crate::span::{self, HighlightKind, Span};

pub static ERR_COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum ReportKind {
	_NOTE_,
	_WARNING_,
	_ERROR_,
	ArgumentParserError,

	// Lexer
	UnexpectedCharacter,
	UnterminatedMultilineComment,
	UnterminatedStringLiteral,
	UnterminatedCharLiteral,
	EmptyCharLiteral,

	// Parser
	UnexpectedToken,
	UnexpectedEOF,
	InvalidNumber,

	// General
	IOError,
	SyntaxError,

	_FATAL_,
}

impl ReportKind {
	pub fn untitled(self) -> Report {
		Report {
			kind:      self,
			title:     None,
			span:      None,
			span_mask: Vec::new(),
			label:     None,
			footers:   None,
		}
	}

	pub fn title<T: Display>(self, title: T) -> Report {
		#[cfg(debug_assertions)]
		assert!(!title.to_string().is_empty(), "use ReportKind::untitled() instead.");
		Report {
			kind:      self,
			title:     Some(title.to_string()),
			span:      None,
			span_mask: Vec::new(),
			label:     None,
			footers:   None,
		}
	}
}

#[derive(Clone)]
pub struct Report {
	kind:      ReportKind,
	title:     Option<String>,
	span:      Option<Span>,
	span_mask: Vec<HighlightKind>,
	label:     Option<String>,
	footers:   Option<Vec<String>>,
}

impl Report {
	pub fn span<T: Into<(Span, Vec<HighlightKind>)>>(mut self, span: T) -> Self {
		let (span, mask) = span.into();
		if self.span.is_none() {
			self.span = Some(span);
		}

		self.span_mask = span::combine(self.span_mask, mask);
		self
	}

	pub fn label<T: Display>(mut self, label: T) -> Self {
		self.label = Some(label.to_string());
		self
	}

	pub fn help<T: Display>(self, help: T) -> Self {
		self.footer(format!("HELP: {help}"))
	}

	pub fn info<T: Display>(self, info: T) -> Self {
		self.footer(format!("INFO: {info}"))
	}

	pub fn note<T: Display>( self, note: T) -> Self {
		self.footer(format!("NOTE: {note}"))
	}

	pub fn as_err<T>(self) -> Result<T> {
		Err(Box::new(self))
	}

	pub fn footer<T: Display>(mut self, text: T) -> Self {
		match self.footers {
			Some(ref mut footers) => footers.push(text.to_string()),
			None => self.footers = Some(vec![text.to_string()]),
		}
		self
	}
}

impl<T> From<Report> for Result<T> {
	#[inline]
	fn from(report: Report) -> Self {
		Err(report.into())
	}
}

impl Display for Report {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		assert!(self.span.is_some() || self.label.is_none());

		if self.kind >= ReportKind::_ERROR_ {
			ERR_COUNT.fetch_add(1, Ordering::Relaxed);
		}

		let (prefix, primary, secondary) = match self.kind {
			k if k > ReportKind::_FATAL_   => ("FATAL", Color::Red,    Color::BrightRed),
			k if k > ReportKind::_ERROR_   => ("ERR",   Color::Red,    Color::BrightRed),
			k if k > ReportKind::_WARNING_ => ("WARN",  Color::Yellow, Color::BrightYellow),
			k if k > ReportKind::_NOTE_    => ("NOTE",  Color::White,  Color::White),
			_ => unreachable!(),
		};

		writeln!(f,
			"{} {}",
			format!("[{prefix}] {:?}:", self.kind).color(primary).bold(),
			self.title.as_ref().unwrap_or(&String::new()),
		)?;

		let mut padding = String::new();
		if let Some(span) = &self.span {
			writeln!(f, " {} {}", "--->".cyan(), self.span.as_ref().unwrap())?;

			padding = format!(
				"{} {} ",
				" ".repeat(span.line_number.to_string().len()),
				"|".cyan().dimmed()
			);

			let Some(line) = CACHE.get(self.span.as_ref().unwrap().filename)
				.lines()
				.nth(self.span.as_ref().unwrap().line_number - 1)
			else {
				return writeln!(f,
					"{padding}{}",
					"Could not fetch line.".color(Color::Red).bold()
				);
			};

			let mut mask_iter = self.span_mask.iter().copied().peekable();
			let mut line_out = String::new();
			let mut span_out = String::new();
			let mut line_chars = line.chars().peekable();

			while let Some(char) = line_chars.peek().copied().or_else(|| mask_iter.peek().map_or(None, |_| Some(' '))) {
				match mask_iter.next().unwrap_or(HighlightKind::Empty) {
					HighlightKind::Empty => {
						span_out.push(' ');
						line_out.push(char);
					},
					HighlightKind::Caret => {
						span_out.push('^');
						line_out.push_str(&char.to_string().color(primary).bold().to_string());
					},
					HighlightKind::Ghost(c) => {
						let mut str = String::from(c);
						span_out.push('^');
						while let Some(HighlightKind::Ghost(c)) = mask_iter.peek().copied() {
							span_out.push('^');
							mask_iter.next();
							str.push(c);
						}

						line_out.push_str(&str.color(Color::Green).bold().to_string());
						continue;
					},
				}
				line_chars.next();
			}

			writeln!(f, "{padding}{line_out}")?;

			writeln!(f,
				"{padding}{} {}",
				span_out.trim_end().color(primary).bold(),
				self.label.as_ref().unwrap_or(&String::new()).color(secondary),
			)?;
		}

		if let Some(footers) = &self.footers {
			for footer in footers {
				writeln!(f, "{}{}", padding, footer.bright_black().italic())?;
			}
		}

		Ok(())
	}
}

impl std::fmt::Debug for Report {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self.kind)
	}
}

pub type Result<T> = std::result::Result<T, Box<Report>>;
