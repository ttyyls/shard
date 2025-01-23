use crate::report::{Result, ReportKind, LogHandler};
use crate::span::Sp;
use crate::parser::ast::{self, Type};

mod hir;
use hir::{Node, ValId};

#[derive(Default)]
pub struct Analyzer<'src> {
	scope: Vec<Scope<'src>>,
}

#[derive(Default)]
struct Scope<'src> {
	idacc:  ValId,
	locals: Vec<(ValId, &'src str, Type<'src>)>, 
}

impl Scope<'_> {
	fn new_id(&mut self) -> ValId {
		self.idacc.0 += 1;
		self.idacc
	}
}

impl<'src> Analyzer<'src> {
	fn push_new_scope(&mut self) {
		self.scope.push(Scope::default());
	}

	fn pop_scope(&mut self) {
		self.scope.pop();
	}

	fn peek_scope_mut(&mut self) -> &mut Scope<'src> {
		self.scope.last_mut().unwrap()
	}

	fn peek_scope(&self) -> &Scope<'src> {
		self.scope.last().unwrap()
	}

	fn get_global_mut(&mut self) -> &mut Scope<'src> {
		self.scope.first_mut().unwrap()
	}

	fn get_global(&self) -> &Scope<'src> {
		self.scope.first().unwrap()
	}

	fn find_matching_descending<F: Fn((ValId, &'src str, &Type)) -> bool>(&self, f: F) 
		-> Option<(ValId, &'src str, Type)> {
		self.scope.iter().rev().find_map(|scope| // TODO: mayhaps dont clone
			scope.locals.iter().rev().find(|(i,n,t)| f((*i,n,t)))).cloned()
	}

	pub fn analyze(ast: Vec<Sp<ast::Node<'src>>>, file: &'static str, handler: &LogHandler) -> Vec<Node<'src>> {
		let mut analyzer = Self {
			scope: vec![Scope::default()],
		};

		ast.into_iter().fold(Vec::new(), |mut acc, node| {
			match analyzer.analyze_root(node) {
				Ok(n)  => acc.push(n),
				Err(e) => {
					handler.log(e.file(file));
					analyzer.scope.truncate(1);
				},
			}; acc
		})
	}

	fn analyze_root(&mut self, node: Sp<ast::Node<'src>>) -> Result<Node<'src>> {
		Ok(match node.elem {
			ast::Node::Func { name, args, ret, attrs, body } 
				if attrs.iter().any(|a| matches!(**a, ast::Attrs::Extern)) => {
				if !body.is_empty() {
					return ReportKind::SyntaxError // TODO: maybe not SyntaxError
						.title("Extern functions cannot have a body")
						.span(node.span)
						.as_err();
				}

				let id = self.peek_scope_mut().new_id();

				let mut nargs = Vec::new();
				let mut fargs = Vec::new();
				for (_, ty) in args {
					if matches!(*ty, Type::Void) {
						return ReportKind::TypeError
							.title("Type 'void' is not allowed as a function argument")
							.help("Remove the arg, or change the type to '*void'")
							.span(ty.span)
							.as_err();
					}
					fargs.push(ty.elem.clone());
					nargs.push(ty);
				}

				let ty = Type::Fn(nargs, ret.clone().map(Box::new));
				self.peek_scope_mut().locals.push((id, name.elem, ty));

				Node::FuncDecl {
					id,
					args: fargs,
					ret:  ret.map_or(Type::Void, |t| t.elem),
				}
			},
			ast::Node::Func { name, args, ret, attrs, body } => {
				let id = self.peek_scope_mut().new_id();
				self.push_new_scope();

				let mut nargs = Vec::new();
				let mut fargs = Vec::new();
				for (n, ty) in args {
					if matches!(*ty, Type::Void) {
						return ReportKind::TypeError
							.title("Type 'void' is not allowed as a function argument")
							.help("Remove the arg, or change the type to '*void'")
							.span(ty.span)
							.as_err();
					}
					let id = self.peek_scope_mut().new_id();
					self.peek_scope_mut().locals.push((id, n.elem, ty.elem.clone()));
					fargs.push((id, ty.elem.clone()));
					nargs.push(ty);
				}

				let ty = Type::Fn(nargs, ret.clone().map(Box::new));
				let ret = ret.map_or(Type::Void, |t| t.elem);

				Node::Func {
					id, ret,
					args:   fargs,
					export: attrs.iter().any(|a| matches!(**a, ast::Attrs::Export)),
					body: todo!(),
					// body:   Vec<Node<'src>>,
				}
			},
			_ => todo!(),
		})
	}

	fn analyze_stmt(&mut self, node: Sp<ast::Node<'src>>) -> Result<Node<'src>> {
		Ok(match node.elem {
			ast::Node::Ret(node) => {
				todo!()
			},
			_ => todo!(),
		})
	}

}
