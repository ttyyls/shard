#![allow(forbidden_lint_groups)]
#![forbid(clippy::complexity,clippy::suspicious,clippy::correctness,clippy::cargo,
	clippy::perf,clippy::pedantic,clippy::nursery)]
#![allow(clippy::style,clippy::restriction,clippy::match_bool,clippy::too_many_lines,
	clippy::single_match_else,clippy::ignored_unit_patterns, clippy::module_name_repetitions,
	clippy::needless_for_each,clippy::derive_partial_eq_without_eq,clippy::missing_const_for_fn,
	clippy::cognitive_complexity,clippy::option_if_let_else,clippy::option_map_unit_fn,
	clippy::similar_names)]
#![allow(dead_code, unused)]

use std::sync::LazyLock;
use std::sync::atomic::Ordering;

use colored::Colorize;

use crate::lexer::Lexer;
use crate::report::{Level, LogHandler, Report, ReportKind, ERR_COUNT};
use crate::fs::CACHE;

mod args;
mod lexer;
mod report;
mod fs;
mod span;

static LOG_HANDLER: LazyLock<LogHandler> = LazyLock::new(LogHandler::new);

fn main() {
	let args = args::Args::parse(std::env::args().skip(1).collect());

	if *args.debug {
		eprintln!("{args:#?}");
	}

	let handler = LogHandler::new();


	let tokens = Lexer::tokenize(*args.file, CACHE.get(*args.file), handler.clone());

	if *args.debug {
		eprintln!("\n{}", "LEXER".bold());
		tokens.iter().for_each(|token| eprintln!("{token:#}"));
	}

	if ERR_COUNT.fetch_add(0, Ordering::Relaxed) > 0 {
		std::process::exit(1);
	}


	handler.terminate();
}
