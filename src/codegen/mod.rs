use std::collections::HashMap;

use crate::parser::ast::{self, Node, Sp, Type as aType, Spannable};
use crate::report::{Result, LogHandler, ReportKind};

mod llvm;
use llvm::{Module, Function, Instr, Value, Type, DataDef, DataAttr, FuncDecl, FuncAttr, ValueKind};

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
	locals: HashMap<ValueKind, (u64, Sp<aType<'src>>)>,
}

impl<'src> Gen<'src> {
	fn push_scope(&mut self) {
		self.scope_stack.push(Scope::default());
	}

	fn pop_scope(&mut self) {
		self.scope_stack.pop();
	}

	fn peek_scope_mut(&mut self) -> &mut Scope<'src> {
		self.scope_stack.last_mut().unwrap()
	}

	fn peek_scope(&self) -> &Scope<'src> {
		self.scope_stack.last().unwrap()
	}

	fn get_global_mut(&mut self) -> &mut Scope<'src> {
		self.scope_stack.first_mut().unwrap()
	}

	fn get_global(&self) -> &Scope<'src> {
		self.scope_stack.first().unwrap()
	}

	fn gen_id(&mut self) -> u64 {
		let id = &mut self.peek_scope_mut().idacc;
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
					self.peek_scope_mut().locals.insert(ValueKind::Temp(name.elem.to_string()), (id, ty));

					out_args.push(Value::new(ValueKind::Temp(id.to_string()), nty));
				}

				let mut attr = FuncAttr::empty();
				if *export { attr |= FuncAttr::EXPORT; }

				let func = Function {
					attr,
					name: &name,
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

	fn gen_stmt(&mut self, ast: Sp<Node<'src>>) -> Result<Instr> {
		Ok(match ast.elem {
			Node::Assign { name, ty, value } => {
				todo!("Move gen_expr to gen_atom and make a new gen_expr"); // FIXME TODO
				//Instr::Assign(Value::Temp(name.elem.to_string()), self.gen_type(&ty)?.expect("todo: implicit type"), Box::new(self.gen_stmt(*value)?))
			}
			Node::Ret(None)       => Instr::Ret(None),
			Node::Ret(Some(expr)) => Instr::Ret(Some(self.gen_expr(&expr)?)),
			Node::FuncCall { name, args } => Instr::Call {
				func: match self.peek_scope().locals.get(&ValueKind::Temp(name.elem.to_string())) {
					Some((i, t)) => Value::new(ValueKind::Temp(i.to_string()), self.gen_type(t)?),
					None => {
						let Some((i, t)) = self.get_global().locals.get(&ValueKind::Global(name.elem.to_string())) else {
							return ReportKind::Undefined
								.title("Call to an undefined function")
								.span(name.span).as_err();
						};
						
						Value::new(ValueKind::Global(i.to_string()), self.gen_type(t)?)
					}
				} ,
				args: args.into_iter().map(|arg| self.gen_expr(&arg)).collect::<Result<_>>()?,
			},
			_ => panic!("GOT: {}", ast),
		})
	}

	fn gen_expr(&mut self, ast: &Sp<Node<'src>>) -> Result<Value> {
		Ok(match &ast.elem {
			Node::UIntLit(n) => Value::new(ValueKind::Const(*n), Type::Ptr),
			Node::StrLit(s)  => {
				// TODO: prevent user from naming shit like this
				let val = ValueKind::Global(format!("__tmp{}", self.gen_id()));

				self.module.data.push(DataDef {
					name:   val.clone(),
					attr:   DataAttr::INTERNAL | DataAttr::CONSTANT,
					value:  Value::new(ValueKind::Str(s.clone()), Type::Array(s.len(), Box::new(Type::Sint(8)))),
				});

				Value::new(val, Type::Ptr)
			},
			_ => todo!(),
		})
	}

	fn gen_type(&self, ty: &Sp<aType>) -> Result<Option<Type>> {
		Ok(Some(match &ty.elem {
			aType::U8  | aType::B8 => Type::Uint(8),
			aType::I8  => Type::Sint(8),

			aType::U16 | aType::B16 => Type::Uint(16),
			aType::I16 => Type::Sint(16),

			aType::U32 | aType::B32 => Type::Uint(32),
			aType::I32 => Type::Sint(32),

			aType::U64 | aType::B64 => Type::Uint(64),
			aType::I64 => Type::Sint(64),

			aType::F32 => Type::Float(32),
			aType::F64 => Type::Float(64),

			aType::Void | aType::Never => return Ok(None),

			aType::Opt(ty) | aType::Mut(ty) => return self.gen_type(&ty),
			aType::Ptr(_) => Type::Ptr,
			aType::Arr(_) => return ReportKind::InvalidType
				.title("Stack arrays are not yet supported")
				.help("Heap allocate :L")
				.span(ty.span).as_err(),
			aType::Ident(i) => match self.typedefs.get(i) {
				Some(TypeDef::Struct { .. } | TypeDef::Enum { .. }) => todo!("llvm composite types"),
				Some(TypeDef::Alias(ty)) => return self.gen_type(ty),
				None => return ReportKind::InvalidType
					.title("Undefined type")
					.span(ty.span).as_err(),
			},
		}))
	}
}
