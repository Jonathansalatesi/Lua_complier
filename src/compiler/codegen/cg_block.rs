use super::super::{
    ast::block::Block,
    codegen::{
        func_info::FuncInfo,
        cg_stat::{cg_ret_stat, cg_stat}
    }
};

pub fn cg_block(fi: &mut FuncInfo, node: &Block) {
    for stat in node.stats.iter() {
        cg_stat(fi, stat);
    }
    
    if node.ret_exps.len() > 0 {
        cg_ret_stat(fi, &node.ret_exps);
    }
}