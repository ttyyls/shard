use std::fmt::{self, Formatter, Display};

#[derive(Default)]
pub struct Module<'src> {
	pub name:  &'src str,
	pub data:  Vec<DataDef>,

	pub decls: Vec<FuncDecl<'src>>,
	pub funcs: Vec<Function<'src>>,
}

pub struct DataDef {
	pub name:  ValueKind,
	pub attr:  DataAttr,
	pub value: Value,
	//pub align: Option<u8>,
}

bitflags::bitflags! {
	pub struct DataAttr: u32 {
		const INTERNAL    = 1;
		const PRIVATE     = 1 << 1;
		const UNNAMEDADDR = 1 << 2;
		const CONSTANT    = 1 << 3;
	}
}

pub struct Function<'src> {
	pub attr:   FuncAttr,
	pub name:   &'src str,
	pub args:   Vec<Value>, // val must be temp
	pub ret:    Option<Type>,
	pub body:   Vec<Instr>,
}

pub struct FuncDecl<'src> {
	pub name: &'src str,
	pub attr: FuncAttr,
	pub args: Vec<Type>,
	pub ret:  Type,
}

bitflags::bitflags! {
	pub struct FuncAttr: u32 {
		const EXPORT = 1;
	}
}

pub enum Instr {
	Assign(ValueKind, Box<Instr>),
	Ret(Option<Value>),
	Call {
		func: Value,
		args: Vec<Value>,
	},
}

pub struct Value {
	pub typ: Option<Type>,
	pub val: ValueKind
}

impl Value {
	pub fn new<T: Into<Option<Type>>>(val: ValueKind, typ: T) -> Self {
		Self { typ: typ.into(), val }
	}
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub enum ValueKind {
	Temp(String),
	Global(String),
	Str(String),
	Const(u64),
}

pub enum Type {
	Sint(u16),
	Uint(u16),
	Float(u16),
	Ptr,
	Array(usize, Box<Type>),
	//Composite(&'src str),
}



impl Display for DataAttr {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if self.contains(Self::INTERNAL)    { write!(f, "internal ")?; }
		if self.contains(Self::PRIVATE)     { write!(f, "private ")?; }
		if self.contains(Self::UNNAMEDADDR) { write!(f, "unnamed_addr ")?; }
		if self.contains(Self::CONSTANT)    { write!(f, "constant ")?; }
		Ok(())
	}
}

impl Display for FuncAttr {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if self.contains(Self::EXPORT) { write!(f, "export ")?; }
		Ok(())
	}
}

impl Display for Module<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		self.data .iter().try_for_each(|d| write!(f, "{d}"))?;
		self.decls.iter().try_for_each(|c| write!(f, "{c}"))?;
		self.funcs.iter().try_for_each(|c| write!(f, "{c}"))
	}
}

impl Display for DataDef {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "{} = {} {}", self.name, self.attr, self.value)
	}
}

impl Display for Function<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "define {} @{}(",
			self.ret.as_ref().map_or(String::new(), ToString::to_string),
			self.name)?;

		self.args.iter().try_for_each(|v| write!(f, "{v}, "))?;
		write!(f, ") {}", self.attr)?;

		writeln!(f, "{{")?;

		self.body.iter().try_for_each(|i| writeln!(f, "\t{i}"))?;
		writeln!(f, "}}")
	}
}

impl Display for FuncDecl<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "declare {} @{}(", self.ret, self.name)?;
		self.args.iter().try_for_each(|t| write!(f, "{t}, "))?;
		write!(f, ")")?;
		self.attr.iter().try_for_each(|a| write!(f, " {a}"))?;
		write!(f, "\n")
	}
}

impl Display for Instr {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			Self::Assign(v, i) => write!(f, "{v} = {i}"),
			Self::Ret(v) => write!(f, "ret {}", v.as_ref()
				.map_or(String::new(), ToString::to_string)),
			Self::Call { func, args } => {
				write!(f, "call {func}(")?;
				args.iter().try_for_each(|v| write!(f, "{v}"))?;
				write!(f, ")")
			},
		}
	}
}

impl Display for Value {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match &self.typ {
			Some(t) => write!(f, "{t} {}", self.val),
			None    => write!(f, "{}", self.val),
		}
	}
}

impl Display for ValueKind {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			Self::Temp(s)   => write!(f, "%{s}"),
			Self::Global(s) => write!(f, "@{s}"),
			Self::Const(c)  => write!(f, "{c}"),
			Self::Str(s)    => write!(f, "s{s:?}"),
		}
	}
}

impl Display for Type {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			Self::Sint(w)     => write!(f, "i{w}"),
			Self::Uint(w)     => write!(f, "u{w}"),
			Self::Float(w)    => write!(f, "f{w}"),
			Self::Ptr         => write!(f, "ptr"),
			Self::Array(n, t) => write!(f, "[{n} x {t}]"),
		}
	}
}
