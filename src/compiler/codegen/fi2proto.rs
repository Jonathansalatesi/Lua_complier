use crate::binchunk::binary_chunk::{Prototype, Upvalue};
use crate::compiler::codegen::func_info::FuncInfo;
use crate::state::lua_value::LuaValue;

pub fn to_proto(fi: &FuncInfo) -> Prototype {
    let mut proto = Prototype {
        source: None,
        lineDefined: 0,
        lastLineDefined: 0,
        numParams: fi.num_params as u8,
        isVararg: 0,
        maxStackSize: fi.max_regs as u8,
        code: fi.insts.clone(),
        constants: get_constants(fi),
        upvalues: get_upvalues(fi),
        protos: to_protos(&fi.sub_funcs),
        lineInfo: vec![],
        locVars: vec![],
        upvalueNames: vec![],
    };
    if fi.is_vararg {
        proto.isVararg = 1;
    }
    proto
}

fn to_protos(fis: &Vec<*mut FuncInfo>) -> Vec<Prototype> {
    let mut protos = vec![];
    for fi in fis.iter() {
        unsafe {
            if !fi.is_null() {
                protos.push(to_proto(&(**fi)));
            }
        }
    }
    protos
}

fn get_constants(fi: &FuncInfo) -> Vec<LuaValue> {
    let mut consts = vec![LuaValue::Nil; fi.constants.len()];
    for i in 0..fi.constants.len() {
        let idx = &fi.constants.values[i];
        let val = &fi.constants.keys[i];
        consts[*idx as usize] = val.clone();
    }
    consts
}

fn get_upvalues(fi: &FuncInfo) -> Vec<Upvalue> {
    let mut upvals = vec![Upvalue{ instack: 0, idx: 0 }; fi.up_values.len()];
    for (_, uv) in fi.up_values.iter() {
        if uv.local_var_slot >= 0 {
            upvals[uv.index as usize] = Upvalue {
                instack: 1,
                idx: uv.local_var_slot as u8,
            };
        } else {
            upvals[uv.index as usize] = Upvalue {
                instack: 0,
                idx: uv.up_val_index as u8,
            };
        }
    }
    upvals
}