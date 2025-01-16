use crate::parser::ast::{self, Node, NodeKind};
use crate::report::{Result, LogHandler};

mod qbe;
use qbe::{Module, Function, Instr, Value, Type};

#[derive(Default)]
pub struct Gen<'src> {
	module: Module<'src>,

	idseed: u64,
}

impl<'src> Gen<'src> {
	pub fn codegen(ast: Vec<Node<'src>>, handler: &LogHandler) -> Module<'src> {
		let mut cgen = Gen::default();

		ast.into_iter().for_each(|global|
			if let Err(e) = cgen.gen_global(global) {
				handler.log(e);
			});

		cgen.module
	}

	fn gen_global(&mut self, ast: Node<'src>) -> Result<()> {
		match ast.kind {
			NodeKind::Func { name, export, args, ret, body } => {
				todo!()
				// let func = Function {
				// 	name, export,
				// 	
				// };
			},
			_ => todo!(),
		}
	}

	fn gen_type(&mut self, ty: Node<'src>) -> Result<Type> {
		let AST::Type(t) = ty.kind
			else { unreachable!() };

	// Word, Long, Single, Double,
	// Byte, HalfWord,
	// Zero, // for zero init

		Ok(match t {
			ast::Type::U8  => Type::Byte,
			// Type::U16 => Type::
			// Type::U32 => "u32",
			// Type::U64 => "u64",
			// Type::I8  => "i8",
			// Type::I16 => "i16",
			// Type::I32 => "i32",
			// Type::I64 => "i64",
			// Type::B8  => "b8",
			// Type::B16 => "b16",
			// Type::B32 => "b32",
			// Type::B64 => "b64",
			// Type::Void => "void",
			// Type::Ptr(i) => return write!(f, "*{i}"),
			// Type::Arr(i) => return write!(f, "[{i}]"),
			// Type::Mut(i) => return write!(f, "mut {i}"),
			// Type::Ident(name) => name,
			_ => todo!(),
		})
	}
}
