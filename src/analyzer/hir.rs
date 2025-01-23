use std::fmt;

use crate::span::Sp;
use crate::parser::ast::Type;

#[derive(Clone, Copy, Default)]
pub struct ValId(pub u64);
impl std::ops::Deref for ValId {
	type Target = u64;
	fn deref(&self) -> &Self::Target 
	{ &self.0 }
}

pub enum Node<'src> {
	Func {
		id:      ValId,
		export:  bool,
		args:    Vec<(ValId, Type<'src>)>,
		ret:     Type<'src>,
		body:    Vec<Node<'src>>,
	},
	FuncDecl {
		id:   ValId,
		args: Vec<Type<'src>>,
		ret:  Type<'src>,
	},
	Assign {
		name:  ValId,
		ty:    Type<'src>,
		value: Var,
	},
	Ret(Option<Var>, Type<'src>),
	FuncCall {
		name: ValId,
		args: Vec<Var>,
	},
	StrLit(String, ValId),
	UIntLit(u64, Type<'src>),
}

pub enum Var {
	Imm(u64),
	Local(ValId),
	Glob(ValId),
}

impl fmt::Display for Node<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			// Self::Func { id, export, args, ret, body } => {
			// 	// writeln!(f, "Func: {}", id)?;
			// 	// writeln!(f, "   Export: {}", export)?;
			// 	// writeln!(f, "   Args: [")?;
			// 	// for (i, (id, typ)) in args.iter().enumerate() {
			// 	// 	write!(f, "      {}: {}", id, typ)?;
			// 	// 	if i != args.len() - 1 {
			// 	// 		writeln!(f, ",")?;
			// 	// 	}
			// 	// }
			// 	// writeln!(f, "]")?;
			// 	// writeln!(f, "   Ret: {}", ret)?;
			// 	// writeln!(f, "   Body: [")?;
			// 	// for (i, node) in body.iter().enumerate() {
			// 	// 	write!(f, "      {}", node)?;
			// 	// 	if i != body.len() - 1 {
			// 	// 		writeln!(f, ",")?;
			// 	// 	}
			// 	// }
			// 	// writeln!(f, "]")
			// }
			// Self::FuncDecl { id, args, ret } => {
			// 	writeln!(f, "FuncDecl: {}", id)?;
			// 	writeln!(f, "   Args: [")?;
			// 	for (i, typ) in args.iter().enumerate() {
			// 		write!(f, "      {}", typ)?;
			// 		if i != args.len() - 1 {
			// 			writeln!(f, ",")?;
			// 		}
			// 	}
			// 	writeln!(f, "]")?;
			// 	writeln!(f, "   Ret: {}", ret)
			// }
			// Self::Assign { name, ty, value } => {
			// 	writeln!(f, "Assign: {} {} = {}", name, ty, value)
			// }
			// Self::Ret(value, ty) => {
			// 	writeln!(f, "Ret: {} {}", value.as_ref().map_or("".to_string(), |v| v.to_string()), ty)
			// }
			// Self::FuncCall { name, args } => {
			// 	writeln!(f, "FuncCall: {}({})", name, args.iter().map(|a| a.to_string()).collect::<Vec<String>>().join(", "))
			// }
			// Self::StrLit(s, id) => {
			// 	writeln!(f, "StrLit: {} {}", s, id)
			// }
			_ => todo!(),
		}
	}
}
