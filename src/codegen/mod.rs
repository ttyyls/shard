use std::collections::HashMap;

use crate::span::Sp;
use crate::parser::ast::{Node, Attrs, Type as aType};
use crate::report::{Result, LogHandler, ReportKind};

mod llvm;
use llvm::{Module, Function, Instr, Val, Type, DataDef, DataAttr, FuncDecl, FuncAttr, ValKind};

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

// prob deref to locals
#[derive(Default)]
struct Scope<'src> {
	idacc:  u64,
	ret:    Option<Type>,
	locals: HashMap<ValKind, (u64, Sp<aType<'src>>)>,
}

impl<'src> Gen<'src> {

	pub fn codegen(file: &'static str, ast: Vec<Sp<Node<'src>>>, handler: &LogHandler) -> Module<'src> {
		todo!("refactor this shit after HIR");
		// let mut cgen = Gen {
		// 	module: Module::default(),
		// 	scope_stack: vec![Scope::default()],
		// 	typedefs: HashMap::new(),
		// };
		//
		// ast.into_iter().for_each(|global|
		// 	if let Err(e) = cgen.gen_global(global) {
		// 		handler.log(e.file(file));
		// 	});
		//
		// cgen.module
	}

	// fn gen_global(&mut self, ast: Sp<Node<'src>>) -> Result<()> {
	// 	match ast.elem {
	// 		Node::Func { name, attrs, args, ret, .. } if attrs.iter().any(|a| matches!(&**a, Attrs::Extern)) => {
	// 			let decl = FuncDecl {
	// 				name: name.elem,
	// 				attr: Self::gen_func_attr(&attrs),
	// 				// TODO: sematntic analysis check for invalid types as args
	// 				args: args.into_iter().map(|(_, ty)| self.gen_type(&ty).transpose().unwrap()).collect::<Result<_>>()?,
	// 				ret:  ret.and_then(|ty| self.gen_type(&ty).transpose()).transpose()?,
	// 			};
	//
	// 			self.module.decls.push(decl);
	// 		},
	// 		Node::Func { name, attrs, args, ret, body } => {
	// 			self.push_scope();
	// 			let ret = ret.and_then(|ty| self.gen_type(&ty).transpose()).transpose()?;
	// 			self.peek_scope_mut().ret = ret.clone();
	//
	// 			let mut out_args = Vec::new();
	// 			for (name, ty) in args {
	// 				let id = self.gen_id();
	//
	// 				let Some(nty) = self.gen_type(&ty)? else {
	// 					return ReportKind::InvalidType
	// 						.title("Type of arg may not be void")
	// 						.span(name.span).as_err();
	// 				};
	// 				self.peek_scope_mut().locals.insert(ValKind::Temp(name.elem.to_string()), (id, ty));
	//
	// 				out_args.push(Val::new(ValKind::Temp(id.to_string()), nty));
	// 			}
	//
	// 			let mut attr = FuncAttr::empty();
	// 			if attrs.iter().any(|a| matches!(&**a, Attrs::Export)) 
	// 				{ attr |= FuncAttr::EXPORT; }
	//
	// 			let func = Function {
	// 				attr, ret,
	// 				name: &name,
	// 				args: out_args,
	// 				body: body.into_iter().map(|stmt| self.gen_stmt(stmt)).collect::<Result<_>>()?,
	// 			};
	//
	// 			self.module.funcs.push(func);
	// 		},
	// 		_ => todo!(),
	// 	}
	// 	Ok(())
	// }
	//
	// fn gen_func_attr(attrs: &[Sp<Attrs>]) -> FuncAttr {
	// 	attrs.iter().fold(FuncAttr::empty(), |acc, a| acc | match &**a {
	// 		Attrs::Export => FuncAttr::EXPORT,
	// 		Attrs::Extern | Attrs::Pub => FuncAttr::empty(),
	// 	})
	// }
	//
	// fn gen_stmt(&mut self, ast: Sp<Node<'src>>) -> Result<Instr> {
	// 	Ok(match ast.elem {
	// 		Node::Assign { name, ty, value } => {
	// 			todo!("Move gen_expr to gen_atom and make a new gen_expr"); // FIXME TODO
	// 			//Instr::Assign(Value::Temp(name.elem.to_string()), self.gen_type(&ty)?.expect("todo: implicit type"), Box::new(self.gen_stmt(*value)?))
	// 		},
	// 		Node::Ret(None)       => Instr::Ret(None),
	// 		Node::Ret(Some(expr)) => {
	// 			let mut val = self.gen_expr(&expr)?;
	// 			val.typ = self.peek_scope().ret.clone();
	// 			Instr::Ret(Some(val))
	// 		},
	//
	// 		// FIXME: move this mess to semantic analysis, that should tell us which func to call
	// 		Node::FuncCall { name, args } => Instr::Call {
	// 			func: match self.peek_scope().locals.get(&ValKind::Temp(name.elem.to_string())) {
	// 				Some((i, t)) => Val::new(ValKind::Temp(i.to_string()), self.gen_type(t)?),
	// 				None => match self.get_global().locals.get(&ValKind::Global(name.elem.to_string())) {
	// 					Some((i, t)) => Val::new(ValKind::Global(i.to_string()), self.gen_type(t)?),
	// 					None => match self.module.decls.iter().find(|decl| decl.name == name.elem) {
	// 						Some(FuncDecl { name, ret, .. }) => Val::new(ValKind::Global(String::from(*name)), ret.clone()),
	// 						None => return ReportKind::Undefined
	// 							.title("Call to an undefined function")
	// 							.span(name.span).as_err(),
	// 					},
	// 				},
	// 			},
	// 			args: args.into_iter().map(|arg| self.gen_expr(&arg)).collect::<Result<_>>()?,
	// 		},
	// 		_ => panic!("GOT: {ast}"),
	// 	})
	// }
	//
	// fn gen_expr(&mut self, ast: &Sp<Node<'src>>) -> Result<Val> {
	// 	Ok(match &ast.elem {
	// 		Node::UIntLit(n) => Val::new(ValKind::Const(*n), Type::Ptr),
	// 		Node::StrLit(s)  => {
	// 			// TODO: prevent user from naming shit like this
	// 			let val = ValKind::Global(format!("__tmp{}", self.gen_id()));
	//
	// 			self.module.data.push(DataDef {
	// 				name:   val.clone(),
	// 				attr:   DataAttr::INTERNAL | DataAttr::CONSTANT,
	// 				value:  Val::new(ValKind::Str(s.clone()), Type::Array(s.len(), Box::new(Type::Int(8)))),
	// 			});
	//
	// 			Val::new(val, Type::Ptr)
	// 		},
	// 		_ => todo!(),
	// 	})
	// }
	//
	// fn gen_type(&self, ty: &Sp<aType>) -> Result<Option<Type>> {
	// 	Ok(Some(match &ty.elem {
	// 		aType::U(i) | aType::B(i) | aType::I(i) => Type::Int(*i),
	// 		aType::F(i) => Type::Float(*i),
	//
	// 		aType::Void | aType::Never => return Ok(None),
	//
	// 		aType::Opt(ty) | aType::Mut(ty) => return self.gen_type(&ty),
	// 		aType::Ptr(_) => Type::Ptr,
	// 		aType::Arr(_) => return ReportKind::InvalidType
	// 			.title("Stack arrays are not yet supported")
	// 			.help("Heap allocate :L")
	// 			.span(ty.span).as_err(),
	// 		aType::Ident(i) => match self.typedefs.get(i) {
	// 			Some(TypeDef::Struct { .. } | TypeDef::Enum { .. }) => todo!("llvm composite types"),
	// 			Some(TypeDef::Alias(ty)) => return self.gen_type(ty),
	// 			None => return ReportKind::InvalidType
	// 				.title("Undefined type")
	// 				.span(ty.span).as_err(),
	// 		},
	// 	}))
	// }
}
