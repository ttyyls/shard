use std::fmt::{self, Formatter, Display};

#[derive(Default)]
pub struct Module<'src> {
	pub name:  &'src str,
	pub data:  Vec<DataDef<'src>>,
	pub funcs: Vec<Function<'src>>,
}

#[derive(Default)]
pub struct DataDef<'src> {
	pub export: bool,
	pub name:   String,
	pub align:  Option<u8>,
	pub items:  Vec<(Type<'src>, Data)>,
}

pub enum Data {
	Str(String),
	Const(u64),
}

#[derive(Default)]
pub struct Function<'src> {
	pub export: bool,
	pub name:   &'src str,
	pub args:   Vec<(Type<'src>, Value)>, // val must be temp
	pub ret:    Option<Type<'src>>,
	pub body:   Vec<Instr<'src>>,
}

pub enum Instr<'src> {
	Assign(Value, Type<'src>, Box<Instr<'src>>),
	Ret(Option<Value>),
	Call {
		func: Value,
		args: Vec<(Value, Type<'src>)>,
	},
}

pub enum Value {
	Temp(String),
	Global(String),
	Const(u64),
}

pub enum Type<'src> {
	Byte, HalfWord, Word, Long, // int
	Single, Double, // flot
	Zero, // for zero init
	Composite(&'src str),
}


impl Display for Module<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		self.data .iter().try_for_each(|d| write!(f, "{d}"))?;
		self.funcs.iter().try_for_each(|c| write!(f, "{c}"))
	}
}

impl Display for DataDef<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if self.export { write!(f, "export ")?; }

		write!(f, "data ${} = ", self.name)?;
		if let Some(a) = self.align { write!(f, "align {a} ")?; }
		write!(f, "{{")?;
		self.items.iter().try_for_each(|(t, d)| write!(f, "{t} {d},"))?;
		writeln!(f, "}}")
	}
}

impl Display for Data {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			Self::Str(s)   => write!(f, "\"{s}\""),
			Self::Const(c) => write!(f, "{c}"),
		}
	}
}

impl Display for Function<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if self.export { write!(f, "export ")?; }

		write!(f, "function {} ${}(",
			self.ret.as_ref().map_or(String::new(), ToString::to_string),
			self.name)?;
		self.args.iter().try_for_each(|(t, v)| write!(f, "{t} {v}, "))?;
		write!(f, ") ")?;

		writeln!(f, "{{")?;
		writeln!(f, "@start")?;

		self.body.iter().try_for_each(|i| writeln!(f, "\t{i}"))?;
		writeln!(f, "}}")
	}
}

impl Display for Instr<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			Self::Assign(v, t, i) => write!(f, "{v} ={t} {i}"),
			Self::Ret(v) => write!(f, "ret {}", v.as_ref()
				.map_or(String::new(), ToString::to_string)),
			Self::Call { func, args } => {
				write!(f, "call {func}(")?;
				args.iter().try_for_each(|(v, t)| write!(f, "{t} {v}, "))?;
				write!(f, ")")
			},
		}
	}
}

impl Display for Value {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			Self::Temp(s)   => write!(f, "%{s}"),
			Self::Global(s) => write!(f, "${s}"),
			Self::Const(c)  => write!(f, "{c}"),
		}
	}
}

impl Display for Type<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "{}", match self {
			Self::Word     => 'w',
			Self::Long     => 'l',
			Self::Single   => 's',
			Self::Double   => 'd',
			Self::Byte     => 'b',
			Self::HalfWord => 'h',
			Self::Zero     => 'z',
			Self::Composite(s) => return write!(f, ":{s}"),
		})
	}
}
