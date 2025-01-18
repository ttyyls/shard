use std::collections::HashMap;

use crate::parser::ast::{self, Node, Sp, Type as aType};
use crate::report::{Result, LogHandler};

mod qbe;
use qbe::{Module, Function, Instr, Value, Type};

pub struct Gen<'src> {
	module: Module<'src>,
	scope_stack: Vec<Scope<'src>>,
}

#[derive(Default)]
struct Scope<'src> {
	idacc:  u64,
	locals: HashMap<&'src str, (u64, ast::Node<'src>)>,
}

impl<'src> Gen<'src> {
	fn push_scope(&mut self) {
		self.scope_stack.push(Scope::default());
	}

	fn pop_scope(&mut self) {
		self.scope_stack.pop();
	}

	fn peek_scope(&mut self) -> &mut Scope<'src> {
		self.scope_stack.last_mut().unwrap()
	}

	fn get_global(&mut self) -> &mut Scope<'src> {
		self.scope_stack.first_mut().unwrap()
	}

	fn gen_id(&mut self) -> u64 {
		let id = &mut self.peek_scope().idacc;
		*id += 1;
		*id - 1
	}

	pub fn codegen(ast: Vec<Sp<Node<'src>>>, handler: &LogHandler) -> Module<'src> {
		let mut cgen = Gen {
			module: Module::default(),
			scope_stack: vec![Scope::default()]
		};

		ast.into_iter().for_each(|global|
			if let Err(e) = cgen.gen_global(global) {
				handler.log(e);
			});

		cgen.module
	}

	fn gen_global(&mut self, ast: Sp<Node<'src>>) -> Result<()> {
		match ast.elem {
			Node::Func { name, export, args, ret, body } => {
				self.push_scope();
				todo!();

				// let mut out_args = Vec::new();
				// for (name, ty) in args {
				// 	let id = self.gen_id();
				// 	self.peek_scope().locals.insert(name, (id, ty));
				//
				// 	let Some(ty) = self.gen_type(ty) else {
				// 		return ReportKind::InvalidType
				// 			.title("Type of arg may not be void")
				// 			.span(name.span).as_err();
				// 	};
				// 	out_args.push((, Value::Temp(id.to_string())));
				// }

				// let func = Function {
				// 	name, export, args: out_args,
				// 	ret: ret.and_then(|ty| self.gen_type(ty)),
				// };
			},
			_ => todo!(),
		}
	}

	fn gen_type(&mut self, ty: Sp<aType<'src>>) -> Option<Type> {
		Some(match ty.elem {
			aType::U8  | aType::B8  | aType::I8  => Type::Byte,
			aType::U16 | aType::B16 | aType::I16 => Type::HalfWord,
			aType::U32 | aType::B32 | aType::I32 => Type::Word,
			aType::U64 | aType::B64 | aType::I64 => Type::Long,

			aType::F32 => Type::Single,
			aType::F64 => Type::Double,

			aType::Void => return None,

			aType::Ptr(_)  => Type::Long,
			aType::Arr(_)  => panic!("unsized"),
			aType::Mut(ty) => return self.gen_type(*ty),
			_ => todo!(),
		})
	}
}
