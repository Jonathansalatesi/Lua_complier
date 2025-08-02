use crate::state::lua_value::LuaValue;
use super::super::{
    ast::{
        exp::*, 
        stat::Stat,
        stat::Stat::*,
    }, 
    codegen::{
        func_info::FuncInfo,
        cg_exp::*,
        cg_block::cg_block
    }
};

pub fn cg_stat(fi: &mut FuncInfo, node: &Stat) {
    match node {
        FuncCallStat( .. ) => {
            cg_func_call_stat(fi, node);
            return;
        },
        BreakStat { .. } => {
            cg_break_stat(fi, node);
            return;
        },
        DoStat { .. } => {
            cg_do_stat(fi, node);
            return;
        },
        RepeatStat { .. } => {
            cg_repeat_stat(fi, node);
            return;
        },
        WhileStat { .. } => {
            cg_whilt_stat(fi, node)
        },
        IfStat { .. } => {
            cg_if_stat(fi, node);
            return;
        },
        ForNumStat { .. } => {
            cg_for_num_stat(fi, node);
            return;
        },
        ForInStat { .. } => {
            cg_for_in_stat(fi, node);
            return;
        },
        AssignStat { .. } => {
            cg_assign_stat(fi, node);
            return;
        },
        LocalVarDeclStat { .. } => {
            cg_local_var_decl_stat(fi, node);
            return;
        },
        LocalFuncDefStat { .. } => {
            cg_local_func_def_stat(fi, node);
            return;
        },
        _ => {
            // LabelStat, GotoStat
            panic!("not supported!");
        }
    }
}

pub fn cg_ret_stat(fi: &mut FuncInfo, exps: &Vec<Exp>) {
    let n_exps = exps.len();

    if n_exps == 0 {
        fi.emit_return(0, 0);
        return;
    }

    if n_exps == 1 {
        if let Exp::NameExp { line: _line, str} = &exps[0] {
            let r = fi.slot_of_local_var(str);
            if r >= 0 {
                fi.emit_return(r, 1);
                return;
            }
        } else if let Exp::FuncCallExp { .. } = &exps[0] {
            let r = fi.alloc_reg();
            cg_tail_call_exp(fi, &exps[0], r);
            fi.free_reg();
            fi.emit_return(r, -1);
            return;
        }
    }
    
    let mult_ret = is_vararg_or_func_call(&exps[n_exps - 1]);
    for (i, exp) in exps.iter().enumerate() {
        let r = fi.alloc_reg();
        if i == n_exps - 1 && mult_ret {
            cg_exp(fi, exp, r, -1);
        } else {
            cg_exp(fi, exp, r, 1);
        }
    }
    fi.free_regs(n_exps as i32);
    
    let a = fi.used_regs;
    if mult_ret {
        fi.emit_return(a, -1);
    } else {
        fi.emit_return(a, n_exps as i32);
    }
}

pub fn is_vararg_or_func_call(exp: &Exp) -> bool {
    match exp { 
        Exp::VarargExp { .. } => true,
        Exp::FuncCallExp { .. } => true,
        _ => false,
    }
}

fn cg_local_func_def_stat(fi: &mut FuncInfo, node: &Stat) {
    if let LocalFuncDefStat {name, exp} = node {
        let r = fi.add_local_var(name);
        cg_func_def_exp(fi, exp.as_ref(), r);
    }
}

fn cg_func_call_stat(fi: &mut FuncInfo, node: &Stat) {
    let r = fi.alloc_reg();
    cg_func_call_exp(fi, node, r, 0);
    fi.free_reg();
}

fn cg_break_stat(fi: &mut FuncInfo, node: &Stat) {
    let pc = fi.emit_jmp(0, 0);
    fi.add_break_jmp(pc);
}

fn cg_do_stat(fi: &mut FuncInfo, node: &Stat) {
    if let DoStat { block } = node {
        fi.enter_scope(false);
        cg_block(fi, block.as_ref());
        fi.close_open_upvals();
        fi.exit_scope();
    }
}

fn cg_whilt_stat(fi: &mut FuncInfo, node: &Stat) {
    if let WhileStat { exp, block } = node {
        let pc_before_exp = fi.pc();
        // step 2
        let r = fi.alloc_reg();
        cg_exp(fi, exp, r, 1);
        fi.free_reg();
        // step 3
        fi.emit_test(r, 0);
        let pc_jmp_to_end = fi.emit_jmp(0, 0);
        // step 4
        fi.enter_scope(true);
        cg_block(fi, block.as_ref());
        fi.close_open_upvals();
        fi.emit_jmp(0, pc_before_exp - fi.pc() - 1);
        fi.exit_scope();
        // step 5
        fi.fix_sBx(pc_jmp_to_end, fi.pc() - pc_jmp_to_end);
    }
}

fn cg_repeat_stat(fi: &mut FuncInfo, node: &Stat) {
    if let RepeatStat { exp, block } = node {
        fi.enter_scope(true);
        
        let pc_before_block = fi.pc();
        cg_block(fi, block.as_ref());
        
        let r = fi.alloc_reg();
        cg_exp(fi, exp, r, 1);
        fi.free_reg();
        
        fi.emit_test(r, 0);
        let tmp_ = fi.get_jmp_argA();
        fi.emit_jmp(tmp_, pc_before_block - fi.pc() - 1);
        fi.close_open_upvals();
        
        fi.exit_scope();
    }
}

fn cg_if_stat(fi: &mut FuncInfo, node: &Stat) {
    if let IfStat { exps, blocks } =  node {
        let mut pc_jmp_to_ends: Vec<i32> = vec![0; exps.len()];
        let mut pc_jmp_to_next_exp = -1;
        for (i, exp) in exps.iter().enumerate() {
            if pc_jmp_to_next_exp >= 0 {
                fi.fix_sBx(pc_jmp_to_next_exp, fi.pc() - pc_jmp_to_next_exp);
            }
            
            let r = fi.alloc_reg();
            cg_exp(fi, exp, r, 1);
            fi.free_reg();
            fi.emit_test(r, 0);
            pc_jmp_to_next_exp = fi.emit_jmp(0, 0);
            
            fi.enter_scope(false);
            cg_block(fi, blocks[i].as_ref());
            fi.close_open_upvals();
            fi.exit_scope();
            if i < exps.len() - 1 {
                pc_jmp_to_ends[i] = fi.emit_jmp(0, 0);
            } else {
                pc_jmp_to_ends[i] = pc_jmp_to_next_exp;
            }
        }
        for pc in pc_jmp_to_ends.iter() {
            fi.fix_sBx(*pc, fi.pc() - *pc);
        }
    }
}

fn cg_for_num_stat(fi: &mut FuncInfo, node: &Stat) {
    if let ForNumStat { line_of_for, line_of_do, var_name, init_exp, limit_exp, step_exp, block } = node {
        fi.enter_scope(true);
        // step 1
        cg_local_var_decl_stat(fi, &LocalVarDeclStat {
            name_list: vec!["(for index)".to_owned(), "(for limit)".to_owned(), "(for step)".to_owned()],
            exp_list: vec![init_exp.clone(), limit_exp.clone(), step_exp.clone()],
            last_line: 0,
        });
        fi.add_local_var(var_name);
        // step 2
        let a = fi.used_regs - 4;
        let pc_for_prep = fi.emit_for_prep(a, 0);
        cg_block(fi, block.as_ref());
        fi.close_open_upvals();
        let pc_for_loop = fi.emit_for_loop(a, 0);
        // step 3
        fi.fix_sBx(pc_for_prep, pc_for_loop - pc_for_prep - 1);
        fi.fix_sBx(pc_for_loop, pc_for_prep - pc_for_loop);
        fi.exit_scope();
    }
}

fn cg_for_in_stat(fi: &mut FuncInfo, node: &Stat) {
    if let ForInStat { line_of_do, name_list, exp_list, block } = node {
        fi.enter_scope(true);
        // step 1
        cg_local_var_decl_stat(fi, &LocalVarDeclStat {
            name_list: vec!["(for generator)".to_owned(), "(for state)".to_owned(), "(for control)".to_owned()],
            exp_list: exp_list.clone(),
            last_line: 0,
        });
        for name in name_list.iter() {
            fi.add_local_var(name);
        }
        // step 2
        let pc_jmp_to_TFC = fi.emit_jmp(0, 0);
        cg_block(fi, block.as_ref());
        fi.close_open_upvals();
        fi.fix_sBx(pc_jmp_to_TFC, fi.pc() - pc_jmp_to_TFC);
        // step 3
        let r_generator = fi.slot_of_local_var("for generator");
        fi.emit_tfor_call(r_generator, name_list.len() as i32);
        fi.emit_tfor_loop(r_generator + 2, pc_jmp_to_TFC - fi.pc() - 1);
        fi.exit_scope();
    }
}

fn cg_local_var_decl_stat(fi: &mut FuncInfo, node: &Stat) {
    if let LocalVarDeclStat { last_line: _last_line, name_list, exp_list } = node {
        let exps = remove_tail_nils(exp_list);
        let n_exps = exps.len();
        let n_names = name_list.len();
        let old_regs = fi.used_regs;
        
        if n_exps == n_names {
            for exp in exps.iter() {
                let a = fi.alloc_reg();
                cg_exp(fi, exp, a, 1);
            }
        } else if n_exps > n_names {
            for (i, exp) in exps.iter().enumerate() {
                let a = fi.alloc_reg();
                if i == n_exps - 1 && is_vararg_or_func_call(exp) {
                    cg_exp(fi, exp, a, 0);
                } else {
                    cg_exp(fi, exp, a, 1);
                }
            }
        } else {
            let mut mult_ret = false;
            for (i, exp) in exps.iter().enumerate() {
                let a = fi.alloc_reg();
                if i == n_exps - 1 && is_vararg_or_func_call(exp) {
                    mult_ret = true;
                    let n = n_names - n_exps + 1;
                    cg_exp(fi, exp, a, n as i32);
                    fi.alloc_regs(n as i32 - 1);
                } else {
                    cg_exp(fi, exp, a, 1);
                }
            }
            if !mult_ret {
                let n = n_names - n_exps;
                let a = fi.alloc_regs(n as i32);
                fi.emit_load_nil(a, n as i32);
            }
        }
        fi.used_regs = old_regs;
        for name in name_list.iter() {
            fi.add_local_var(name);
        }
    }
}

pub fn remove_tail_nils(exps: &Vec<Exp>) -> Vec<Exp> {
    for n in (0..=(exps.len() - 1)).rev() {
        if let Exp::NilExp { .. } = exps[n] {} else if let Exp::EmptyExp = exps[n] {} else {
            return exps[0..=n].to_vec();
        }
    }
    vec![]
}

fn cg_assign_stat(fi: &mut FuncInfo, node: &Stat) {
    if let AssignStat { last_line: _, var_list, exp_list } = node {
        let exps = remove_tail_nils(exp_list);
        let n_exps = exps.len();
        let n_vars = var_list.len();
        
        let mut t_regs = vec![0; n_vars];
        let mut k_regs = vec![0; n_vars];
        let mut v_regs = vec![0; n_vars];
        let old_regs = fi.used_regs;
        
        for (i, exp) in var_list.iter().enumerate() {
            if let Exp::TableAccessExp { last_line: _, prefix_exp, key_exp } = exp {
                t_regs[i] = fi.alloc_reg();
                cg_exp(fi, prefix_exp, t_regs[i], 1);
                k_regs[i] = fi.alloc_reg();
                cg_exp(fi, key_exp, k_regs[i], 1);
            }
        }
        for i in 0..n_vars {
            v_regs[i] = fi.used_regs + i as i32;
        }
        
        if n_exps >= n_vars {
            for (i, exp) in exps.iter().enumerate() {
                let a = fi.alloc_reg();
                if i >= n_vars && i == n_exps - 1 && is_vararg_or_func_call(exp) {
                    cg_exp(fi, exp, a, 0);
                } else {
                    cg_exp(fi, exp, a, 1);
                }
            }
        } else {    // n_vars > n_exps
            let mut mult_ret = false;
            for (i, exp) in exps.iter().enumerate() {
                let a = fi.alloc_reg();
                if i == n_exps - 1 && is_vararg_or_func_call(exp) {
                    mult_ret = true;
                    let n = n_vars - n_exps + 1;
                    cg_exp(fi, exp, a, n as i32);
                    fi.alloc_regs(n as i32 - 1);
                } else {
                    cg_exp(fi, exp, a, 1);
                }
            }
            if !mult_ret {
                let n = n_vars - n_exps;
                let a = fi.alloc_regs(n as i32);
                fi.emit_load_nil(a, n as i32);
            }
        }
        
        for (i, exp) in var_list.iter().enumerate() {
            if let Exp::NameExp { line, str } = exp {
                let var_name = str;
                let a = fi.slot_of_local_var(var_name);
                if a >= 0 {
                    fi.emit_move(a, v_regs[i]);
                } else {
                    let b = fi.index_of_upVal(var_name);
                    if b >= 0 {
                        fi.emit_set_upval(v_regs[i], b);
                    } else {
                        let a = fi.index_of_upVal("_ENV");
                        let b = 0x100 + fi.index_of_constant(&LuaValue::Str(var_name.to_owned()));
                        fi.emit_set_tab_up(a, b, v_regs[i]);
                    }
                }
            } else {
                fi.emit_set_table(t_regs[i], k_regs[i], v_regs[i]);
            }
        }
        fi.used_regs = old_regs;
    }
}