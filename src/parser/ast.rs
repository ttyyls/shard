use std::fmt::{self, Display};
use crate::span::Span;

use colored::Colorize;

pub struct Sp<T> {
	pub span: Span,
	pub elem: T,
}

impl<T> std::ops::Deref for Sp<T> {
	type Target = T;
	fn deref(&self) -> &Self::Target 
		{ &self.elem }
}

impl<T> std::ops::DerefMut for Sp<T> {
	fn deref_mut(&mut self) -> &mut Self::Target 
		{ &mut self.elem }
}

pub trait Spannable {
	fn span(self, span: Span) -> Sp<Self> where Self: Sized
		{ Sp { span, elem: self } }
}

impl<T> Spannable for T {}

pub enum Node<'src> {
	DBG,
	Func {
		name:   Sp<&'src str>,
		attrs:  Vec<Sp<Attrs>>,
		args:   Vec<(Sp<&'src str>, Sp<Type<'src>>)>,
		ret:    Option<Sp<Type<'src>>>,
		body:   Vec<Sp<Node<'src>>>
	},
	Assign {
		name: Sp<&'src str>,
		ty: Sp<Type<'src>>,
		value: Box<Sp<Node<'src>>>
	},
	Ret(Option<Box<Sp<Node<'src>>>>),
	FuncCall {
		name: Sp<&'src str>,
		args: Vec<Sp<Node<'src>>>,
	},
	StrLit(String),
	UIntLit(u64),
}

pub enum Attrs {
	Export,
	Extern,
	Pub,
}

pub enum Type<'src> {
	U(u16), I(u16), B(u16), F(u16),
	Void, Never,
	Opt(Box<Sp<Type<'src>>>),
	Ptr(Box<Sp<Type<'src>>>),
	Arr(Box<Sp<Type<'src>>>),
	Mut(Box<Sp<Type<'src>>>),
	Ident(&'src str),
}

impl<T: Display> Display for Sp<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{} {}", self.span, self.elem)
	}
}

// TODO: better display for this shit prob
impl Display for Node<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::DBG => write!(f, "DBG"),
			Self::Func { name, attrs, args, ret, body } => {
				writeln!(f, "Func: {}", name.to_string().blue())?;

				write!(f, "Attrs: [")?;
				attrs.iter().try_for_each(|a| write!(f, "{a} "))?;
				writeln!(f, "]")?;

				write!(f, "   Args: [")?;
				for (i, (name, typ)) in args.iter().enumerate() {
					write!(f, "{name}: {typ}")?;
					if i != args.len() - 1 {
						write!(f, ", ")?;
					}
				}
				write!(f, "]\n   Ret: {}\n", ret.as_ref().map_or("void".to_string(), ToString::to_string))?;
				for stmt in body {
					writeln!(f, "      {stmt}")?;
				}
				Ok(())
			},
			Self::Assign { name, value, ty } => {
				write!(f, "Assignment: {}: {} = {}", name.to_string().blue(), ty, value)
			},
			Self::Ret(expr) => match expr {
				Some(expr) => write!(f, "Ret: {expr}"),
				None => write!(f, "Ret"),
			},
			Self::FuncCall { name, args } => {
				write!(f, "FuncCall: {}(", name.to_string().blue())?;
				for (i, arg) in args.iter().enumerate() {
					write!(f, "{arg}")?;
					if i != args.len() - 1 {
						write!(f, ", ")?;
					}
				}
				write!(f, ")")
			},
			Self::StrLit(s)  => write!(f, "StrLit: {}", format!("{s:?}").green()),
			Self::UIntLit(i) => write!(f, "UIntLit: {}", i.to_string().green()),
		}
	}
}

impl Display for Type<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", match self {
			Self::U(i)   => format!("u{i}"),
			Self::I(i)   => format!("i{i}"),
			Self::B(i)   => format!("b{i}"),
			Self::F(i)   => format!("f{i}"),
			Self::Void   => String::from("void"),
			Self::Never  => String::from("never"),
			Self::Opt(i) => format!("opt {i}"),
			Self::Ptr(i) => format!("*{i}"),
			Self::Arr(i) => format!("[{i}]"),
			Self::Mut(i) => format!("mut {i}"),
			Self::Ident(name) => String::from(*name),
		}.red())
	}
}

impl Display for Attrs {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", match self {
			Self::Export => "export",
			Self::Extern => "extern",
			Self::Pub    => "pub",
		}.yellow().dimmed())
	}
}
