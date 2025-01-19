use std::collections::HashMap;

use crate::parser::ast::{self, Node, Sp, Type as aType, Spannable};
use crate::report::{Result, LogHandler, ReportKind};

mod qbe;
use qbe::{Module, Function, Instr, Value, Type, DataDef, Data};

pub struct Gen<'src> {
	module:      Module<'src>,

	scope_stack: Vec<Scope<'src>>,
	typedefs:    HashMap<&'src str, TypeDef<'src>>,
}

// TODO: impl typedef creation
pub enum TypeDef<'src> {
	Struct(Vec<(&'src str, Sp<aType<'src>>)>),
	Enum(Vec<(&'src str, Option<Sp<aType<'src>>>)>),
	Alias(Sp<aType<'src>>),
}

#[derive(Default)]
struct Scope<'src> {
	idacc:  u64,
	locals: HashMap<&'src str, (u64, Sp<aType<'src>>)>,
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

	pub fn codegen(file: &'static str, ast: Vec<Sp<Node<'src>>>, handler: &LogHandler) -> Module<'src> {
		let mut cgen = Gen {
			module: Module::default(),
			scope_stack: vec![Scope::default()],
			typedefs: HashMap::new(),
		};

		ast.into_iter().for_each(|global|
			if let Err(e) = cgen.gen_global(global) {
				handler.log(e.file(file));
			});

		cgen.module
	}

	fn gen_global(&mut self, ast: Sp<Node<'src>>) -> Result<()> {
		match ast.elem {
			Node::Func { name, export, args, ret, body } => {
				self.push_scope();

				let mut out_args = Vec::new();
				for (name, ty) in args {
					let id = self.gen_id();

					let Some(nty) = self.gen_type(&ty)? else {
						return ReportKind::InvalidType
							.title("Type of arg may not be void")
							.span(name.span).as_err();
					};
					self.peek_scope().locals.insert(&name, (id, ty));

					out_args.push((nty, Value::Temp(id.to_string())));
				}

				let func = Function {
					name: &name,
					export: *export,
					args: out_args,
					ret:  ret.and_then(|ty| self.gen_type(&ty).transpose()).transpose()?,
					body: body.into_iter().map(|stmt| self.gen_stmt(stmt)).collect::<Result<_>>()?,
				};

				self.module.funcs.push(func);
			},
			_ => todo!(),
		}
		Ok(())
	}

	fn gen_stmt(&mut self, ast: Sp<Node<'src>>) -> Result<Instr<'src>> {
		Ok(match ast.elem {
			Node::Ret(None)       => Instr::Ret(None),
			Node::Ret(Some(expr)) => Instr::Ret(Some(self.gen_expr(&expr)?.0)),
			Node::FuncCall { name, args } => {
				Instr::Call {
					// TODO: check if in scope
					func: self.peek_scope().locals.get(&*name).map_or_else(
						|| Value::Global(name.elem.to_string()),
						|(i, _)| Value::Temp(i.to_string())),
					args: args.into_iter().map(|arg| self.gen_expr(&arg)).collect::<Result<_>>()?,
				}
			},
			_ => todo!(),
		})
	}

	fn gen_expr(&mut self, ast: &Sp<Node<'src>>) -> Result<(Value, Type<'src>)> {
		Ok(match ast.elem {
			Node::UIntLit(n) => (Value::Const(n), Type::Long),
			Node::StrLit(s)  => {
				// TODO: prevent user from naming shit like this
				let name = format!("__tmp{}", self.gen_id());

				let mut items = Vec::new();
				let mut chars = s.chars();

				while let Some(c) = chars.next() {
					if matches!(c, '\x00'..='\x1f') {
						items.push((Type::Byte, Data::Const(u64::from(c as u8))));
						continue;
					}

					match items.last_mut() {
						Some((_, Data::Str(s))) => s.push(c),
						_ => items.push((Type::Byte, Data::Str(c.to_string()))),	
					}
				}

				self.module.data.push(DataDef {
					name:   name.clone(),
					export: false,
					align:  None,
					items,
				});

				(Value::Global(name), Type::Long)
			},
			_ => todo!(),
		})
	}

	fn gen_type(&self, ty: &Sp<aType<'src>>) -> Result<Option<Type<'src>>> {
		Ok(Some(match &ty.elem {
			aType::U8  | aType::B8  | aType::I8  => Type::Byte,
			aType::U16 | aType::B16 | aType::I16 => Type::HalfWord,
			aType::U32 | aType::B32 | aType::I32 => Type::Word,
			aType::U64 | aType::B64 | aType::I64 => Type::Long,

			aType::F32 => Type::Single,
			aType::F64 => Type::Double,

			aType::Void | aType::Never => return Ok(None),

			aType::Opt(_) | aType::Ptr(_) => Type::Long,
			aType::Arr(_) => return ReportKind::InvalidType
				.title("Stack arrays are not yet supported")
				.help("Heap allocate :L")
				.span(ty.span).as_err(),
			aType::Mut(ty) => return self.gen_type(&ty),
			aType::Ident(i) => match self.typedefs.get(i) {
				Some(TypeDef::Struct { .. } | TypeDef::Enum { .. }) => Type::Composite(i),
				Some(TypeDef::Alias(ty)) => return self.gen_type(ty),
				None => return ReportKind::InvalidType
					.title("Undefined type")
					.span(ty.span).as_err(),
			},
		}))
	}
}
