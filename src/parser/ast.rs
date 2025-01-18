use std::fmt;
use crate::span::Span;

pub struct Node<'src> {
	pub kind: NodeKind<'src>,
	pub span: Span,
}

pub enum NodeKind<'src> {
	DBG,
	Func {
		name: &'src str, 
		export: bool, // TODO: move separate struc
		args: Vec<(&'src str, Type<'src>)>,
		ret:  Option<Type<'src>>,
		body: Vec<Node<'src>>,
	},
	Type(Type<'src>),
	Ret(Box<Node<'src>>),
	FuncCall {
		name: &'src str,
		args: Vec<Node<'src>>,
	},
	StrLit(&'src str),
	UIntLit(u64),
}

pub enum Type<'src> {
	U8, U16, U32, U64,
	I8, I16, I32, I64,
	B8, B16, B32, B64,
	F32, F64,
	Void, Never,
	Opt(Box<Type<'src>>),
	Ptr(Box<Type<'src>>),
	Arr(Box<Type<'src>>),
	Mut(Box<Type<'src>>),
	Ident(&'src str),
}

impl fmt::Display for Node<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{} {}", self.span, self.kind)
	}
}

// TODO: better display for this shit prob
impl fmt::Display for NodeKind<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			NodeKind::DBG => write!(f, "DBG"),
			NodeKind::Func { name, export, args, ret, body } => {
				write!(f, "Func: {name} {}\n", if *export { "export" } else { "" })?;
				write!(f, "  Args: [")?;
				for (i, (name, typ)) in args.iter().enumerate() {
					write!(f, "{name}: {typ}")?;
					if i != args.len() - 1 {
						write!(f, ", ")?;
					}
				}
				write!(f, "]\n  Ret: {}\n", ret.as_ref().map_or("void".to_string(), ToString::to_string))?;
				for stmt in body {
					writeln!(f, "    {stmt}")?;
				}
				Ok(())
			}
			NodeKind::Type(t) => write!(f, "Type: {t}"),
			NodeKind::Ret(expr) => write!(f, "Ret: {expr}"),
			NodeKind::FuncCall { name, args } => {
				write!(f, "FuncCall: {name}(")?;
				for (i, arg) in args.iter().enumerate() {
					write!(f, "{arg}")?;
					if i != args.len() - 1 {
						write!(f, ", ")?;
					}
				}
				write!(f, ")")
			},
			NodeKind::StrLit(s)  => write!(f, "StrLit: \"{s}\""),
			NodeKind::UIntLit(i) => write!(f, "UIntLit: {i}"),
		}
	}
}

impl fmt::Display for Type<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", match self {
			Type::U8  => "u8",
			Type::U16 => "u16",
			Type::U32 => "u32",
			Type::U64 => "u64",
			Type::I8  => "i8",
			Type::I16 => "i16",
			Type::I32 => "i32",
			Type::I64 => "i64",
			Type::B8  => "b8",
			Type::B16 => "b16",
			Type::B32 => "b32",
			Type::B64 => "b64",
			Type::F32 => "f32",
			Type::F64 => "f64",
			Type::Void => "void",
			Type::Never => "never",
			Type::Opt(i) => return write!(f, "opt {i}"),
			Type::Ptr(i) => return write!(f, "*{i}"),
			Type::Arr(i) => return write!(f, "[{i}]"),
			Type::Mut(i) => return write!(f, "mut {i}"),
			Type::Ident(name) => name,
		})
	}
}
