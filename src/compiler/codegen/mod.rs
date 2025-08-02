use crate::binchunk::binary_chunk::Prototype;
use crate::compiler::ast::block::Block;
use crate::compiler::ast::exp::Exp::FuncDefExp;
use std::rc::Rc;
use crate::compiler::codegen::cg_exp::cg_func_def_exp;
use crate::compiler::codegen::fi2proto::to_proto;
use crate::compiler::codegen::func_info::FuncInfo;
use crate::compiler::parser::parse;

pub mod func_info;
pub mod cg_block;
pub mod cg_stat;
mod cg_exp;
mod fi2proto;

fn gen_proto(chunk: Rc<Block>) -> Prototype {
    let fd = FuncDefExp {
        line: 0,
        last_line: 0,
        par_list: vec![],
        is_vararg: true,
        block: chunk.clone(),
    };
    let mut fi = FuncInfo::new(&fd);
    fi.add_local_var("_ENV");
    cg_func_def_exp(&mut fi, &fd, 0);
    unsafe {
        to_proto(&*fi.sub_funcs[0])
    }
}

pub fn compile(chunk: String, chunk_name: String) -> Prototype {
    let ast = parse(chunk, chunk_name);
    // println!("{:#?}", *ast);
    gen_proto(ast.clone())
}