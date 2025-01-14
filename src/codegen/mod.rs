use crate::parser::ast::{self, AST};
use crate::report::{Result, LogHandler};

mod qbe;
use qbe::{Module, Function, Instr, Value, Type};

#[derive(Default)]
pub struct Gen<'src> {
	mo: Module<'src>,
}

impl<'src> Gen<'src> {
	pub fn codegen(ast: AST<'src>, handler: &LogHandler) -> Module<'src> {
		let mut cgen = Gen::default();

		let AST::Module(name, globals) = ast 
			else { unreachable!() };

		cgen.mo.name = name;

		globals.into_iter().for_each(|global|
			if let Err(e) = cgen.gen_global(global) {
				handler.log(e);
			});

		cgen.mo
	}

	fn gen_global(&mut self, ast: AST<'src>) -> Result<()> {
		todo!()
	}
}
