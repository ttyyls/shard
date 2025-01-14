use std::fmt;

// TODO !!!!!! @fami-fish pls
// pub struct Node<'src> {
// 	pub kind: NodeKind<'src>,
// 	pub span: Span,
// }

pub enum AST<'src> {
	DBG,
	Module(&'src str, Vec<AST<'src>>),
	Func {
		name: &'src str, 
		linkage: bool, // TODO: move separate struc; true = export
		args: Vec<(&'src str, Type<'src>)>,
		ret:  Option<Type<'src>>,
		body: Vec<AST<'src>>,
	},
	Type(Type<'src>),
	Ret(Box<AST<'src>>),
	FuncCall {
		name: &'src str,
		args: Vec<AST<'src>>,
	},
	StrLit(&'src str),
	UIntLit(u64),
}

pub enum Type<'src> {
	U8, U16, U32, U64,
	I8, I16, I32, I64,
	B8, B16, B32, B64,
	Void,
	Ptr(Box<Type<'src>>),
	Arr(Box<Type<'src>>),
	Mut(Box<Type<'src>>),
	Ident(&'src str),
}


impl fmt::Display for AST<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			AST::DBG => write!(f, "DBG"),
			AST::Module(name, body) => {
				write!(f, "Module: {name}\n")?;
				for item in body {
					writeln!(f, "  {item}")?;
				}
				Ok(())
			}
			AST::Func { name, linkage, args, ret, body } => {
				write!(f, "Func: {name} (linkage: {linkage})\n")?;
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
			AST::Type(t) => write!(f, "Type: {t}"),
			AST::Ret(expr) => write!(f, "Ret: {expr}"),
			AST::FuncCall { name, args } => {
				write!(f, "FuncCall: {name}(")?;
				for (i, arg) in args.iter().enumerate() {
					write!(f, "{arg}")?;
					if i != args.len() - 1 {
						write!(f, ", ")?;
					}
				}
				write!(f, ")")
			},
			AST::StrLit(s)  => write!(f, "StrLit: {s}"),
			AST::UIntLit(i) => write!(f, "UIntLit: {i}"),
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
			Type::Void => "void",
			Type::Ptr(i) => return write!(f, "*{i}"),
			Type::Arr(i) => return write!(f, "[{i}]"),
			Type::Mut(i) => return write!(f, "mut {i}"),
			Type::Ident(name) => name,
		})
	}
}

