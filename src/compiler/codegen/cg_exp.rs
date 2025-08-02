use std::rc::Rc;
use crate::state::lua_value::LuaValue;
use crate::vm::opcodes::*;
use super::super::{
    ast::{
        exp::Exp::{self, *},
        stat::Stat
    },
    codegen::{
        func_info::FuncInfo,
        cg_stat::*,
        cg_block::cg_block
    },
    lexer::token::*
};

pub fn cg_tail_call_exp(fi: &mut FuncInfo, exp: &Exp, a: i32) {
    let n_args = prep_func_call(fi, exp, a);
    fi.emit_tail_call(a, n_args);
}

pub fn cg_exp(fi: &mut FuncInfo, exp: &Exp, a: i32, n: i32) {
    match exp {
        NilExp { .. } => fi.emit_load_nil(a, n),
        FalseExp { .. } => fi.emit_load_bool(a, 0, 0),
        TrueExp { .. } => fi.emit_load_bool(a, 1, 0),
        IntegerExp { line, val } => fi.emit_load_K(a, &LuaValue::Integer(*val)),
        FloatExp { line, val } => fi.emit_load_K(a, &LuaValue::Number(*val)),
        StringExp { line, str } => fi.emit_load_K(a, &LuaValue::Str(str.to_owned())),
        ParensExp { exp: exp0 } => cg_exp(fi, exp0.as_ref(), a, 1),
        VarargExp { .. } => cg_vararg_exp(fi, exp, a, n),
        FuncDefExp { .. } => cg_func_def_exp(fi, exp, a),
        TableConstructorExp { .. } => cg_table_constructor_exp(fi, exp, a),
        UnopExp { .. } => cg_unop_exp(fi, exp, a),
        BinopExp { .. } => cg_binop_exp(fi, exp, a),
        ConcatExp { .. } => cg_concat_exp(fi, exp, a),
        NameExp { .. } => cg_name_exp(fi, exp, a),
        TableAccessExp { .. } => cg_table_access_exp(fi, exp, a),
        FuncCallExp { .. } => cg_func_call_exp_(fi, exp, a, n),
        _ => {
            panic!("no such expression.");
        }
    }
}

fn cg_vararg_exp(fi: &mut FuncInfo, _exp: &Exp, a: i32, n: i32) {
    if !fi.is_vararg {
        panic!(r"cannot use '...' outside a vararg function");
    }
    fi.emit_vararg(a, n);
}

pub fn cg_func_def_exp(fi: &mut FuncInfo, node: &Exp, a: i32) {
    if let Exp::FuncDefExp { line, last_line, par_list, is_vararg, block } = node {
        let ptr_sub_FI = FuncInfo::new_ptr(fi as *mut FuncInfo, node);
        // let mut sub_FI = FuncInfo::new(node);
        unsafe {
            let mut_sub_FI = &mut *ptr_sub_FI;
            // mut_sub_FI.ref_set_parent(fi);
            for param in par_list.iter() {
                mut_sub_FI.add_local_var(param);
            }
            cg_block(mut_sub_FI, block.as_ref());
            mut_sub_FI.exit_scope();
            mut_sub_FI.emit_return(0, 0);

            // fi.sub_funcs.push(Rc::from(sub_FI));
            let bx = fi.sub_funcs.len() - 1;
            fi.emit_closure(a, bx as i32);
        }
    }
}

fn cg_table_constructor_exp(fi: &mut FuncInfo, node: &Exp, a: i32) {
    if let Exp::TableConstructorExp { line, last_line, key_exps, val_exps } = node {
        let mut n_arr = 0;
        for key_exp in key_exps.iter() {
            if let EmptyExp = key_exp {
                n_arr += 1;
            }
        }
        
        let n_exps = key_exps.len();
        let mult_ret = n_exps > 0 && is_vararg_or_func_call(&val_exps[n_exps - 1]);
        
        fi.emit_new_table(a, n_arr, n_exps as i32 - n_arr);
        
        let mut arr_idx = 0;
        for (i, key_exp) in key_exps.iter().enumerate() {
            let val_exp = &val_exps[i];
            if let EmptyExp = key_exp {
                arr_idx += 1;
                let tmp = fi.alloc_reg();
                if i == n_exps - 1 && mult_ret {
                    cg_exp(fi, val_exp, tmp, -1);
                } else {
                    cg_exp(fi, val_exp, tmp, 1);
                }
                
                if arr_idx % 50 == 0 || arr_idx == n_arr {  // LFIELDS_PER_FLUSH
                    let mut n = arr_idx % 50;
                    if n == 0 {
                        n = 50;
                    }
                    let c = (arr_idx - 1) / 50 + 1;
                    fi.free_regs(n);
                    if i == n_exps - 1 && mult_ret {
                        fi.emit_set_list(a, 0, c);
                    } else {
                        fi.emit_set_list(a, n, c);
                    }
                }
                continue;
            }
            
            let b = fi.alloc_reg();
            cg_exp(fi, key_exp, b, 1);
            let c = fi.alloc_reg();
            cg_exp(fi, val_exp, c, 1);
            fi.free_regs(2);
            
            fi.emit_set_table(a, b, c);
        }
    }
}

fn cg_unop_exp(fi: &mut FuncInfo, node: &Exp, a: i32) {
    if let UnopExp { line, op, exp } =  node {
        let b = fi.alloc_reg();
        cg_exp(fi, exp, b, 1);
        fi.emit_unary_op(*op, a, b);
        fi.free_reg();
    }
}

fn cg_concat_exp(fi: &mut FuncInfo, node: &Exp, a: i32) {
    if let ConcatExp { line, exps } = node {
        for sub_exp in exps.iter() {
            let a = fi.alloc_reg();
            cg_exp(fi, sub_exp, a, 1);
        }
        
        let c = fi.used_regs - 1;
        let b = c - exps.len() as i32 + 1;
        fi.free_regs(c - b + 1);
        fi.emit_ABC(OP_CONCAT as i32, a, b, c);
    }
}

fn cg_binop_exp(fi: &mut FuncInfo, node: &Exp, a: i32) {
    if let BinopExp { line, op, exp1, exp2 } = node {
        match *op { 
            TOKEN_OP_AND | TOKEN_OP_OR => {
                let b = fi.alloc_reg();
                cg_exp(fi, exp1.as_ref(), b, 1);
                fi.free_reg();
                if *op == TOKEN_OP_AND {
                    fi.emit_test_set(a, b, 0);
                } else {
                    fi.emit_test_set(a, b, 1);
                }
                let pc_of_jmp = fi.emit_jmp(0, 0);
                let b = fi.alloc_reg();
                cg_exp(fi, exp2, b, 1);
                fi.free_reg();
                fi.emit_move(a, b);
                fi.fix_sBx(pc_of_jmp, fi.pc() - pc_of_jmp);
            },
            _ => {
                let b = fi.alloc_reg();
                cg_exp(fi, exp1, b, 1);
                let c = fi.alloc_reg();
                cg_exp(fi, exp2, c, 1);
                fi.emit_binary_op(*op, a, b, c);
                fi.free_regs(2);
            }
        }
    }
}

fn cg_name_exp(fi: &mut FuncInfo, node: &Exp, a: i32) {
    if let NameExp { line, str } = node {
        let r = fi.slot_of_local_var(str);
        if r >= 0 {
            fi.emit_move(a, r);
        } else {
            let idx = fi.index_of_upVal(str);
            if idx >= 0 {
                fi.emit_get_upval(a, idx);
            } else {    // x => _ENV['x']
                let ta_exp = TableAccessExp {
                    prefix_exp: Box::new(NameExp {
                        line: 0,
                        str: "_ENV".to_owned(),
                    }),
                    key_exp: Box::new(StringExp {
                        line: 0,
                        str: str.to_owned(),
                    }),
                    last_line: 0,
                };
                cg_table_access_exp(fi, &ta_exp, a);
            }
        }
    }
}

pub fn cg_table_access_exp(fi: &mut FuncInfo, node: &Exp, a: i32) {
    if let TableAccessExp { last_line, prefix_exp, key_exp } = node {
        let b = fi.alloc_reg();
        cg_exp(fi, prefix_exp, b, 1);
        let c = fi.alloc_reg();
        cg_exp(fi, key_exp, c, 1);
        fi.emit_get_table(a, b, c);
        fi.free_regs(2);
    }
}

fn cg_func_call_exp_(fi: &mut FuncInfo, exp: &Exp, a: i32, n: i32) {
    let n_args = prep_func_call(fi, exp, a);
    fi.emit_call(a, n_args, n);
}

pub fn cg_func_call_exp(fi: &mut FuncInfo, node: &Stat, a: i32, n: i32) {
    if let Stat::FuncCallStat(exp) = node {
        let n_args = prep_func_call(fi, exp, a);
        fi.emit_call(a, n_args, n);
    }
}

fn prep_func_call(fi: &mut FuncInfo, node: &Exp, a: i32) -> i32 {
    if let FuncCallExp { line, last_line, prefix_exp, name_exp, args } = node {
        let mut n_args = args.len();
        let mut last_arg_is_vararg_or_func_call = false;
        
        cg_exp(fi, prefix_exp, a, 1);
        if let EmptyExp = name_exp.as_ref() {} else if let NameExp { line, str } = name_exp.as_ref() {
             let c = 0x100 + fi.index_of_constant(&LuaValue::Str(str.to_owned()));
            fi.emit_self(a, a, c);
        }
        
        for (i, arg) in args.iter().enumerate() {
            let tmp = fi.alloc_reg();
            if i == n_args - 1 && is_vararg_or_func_call(arg) {
                last_arg_is_vararg_or_func_call = true;
                cg_exp(fi, arg, tmp, -1);
            } else {
                cg_exp(fi, arg, tmp, 1);
            }
        }
        
        fi.free_regs(n_args as i32);
        if let EmptyExp = name_exp.as_ref() {} else {
            n_args += 1;
        }
        if last_arg_is_vararg_or_func_call {
            n_args -= 1;
        }
        return n_args as i32;
    }
    panic!("Exp is not FuncCallExp.");
}


#[cfg(test)]
mod tests {
    use std::alloc::{dealloc, Layout};
    use std::ptr::null_mut;

    struct Node {
        val: i32,
        prev: *mut Node,
        next: *mut Node,
    }
    
    impl Node {
        fn new(v: i32) -> Self {
            Self {
                val: v,
                prev: null_mut(),
                next: null_mut(),
            }
        }
        
        fn new_after(v: i32, prev: *mut Node) -> Self {
            Self {
                val: v,
                prev,
                next: null_mut(),
            }
        }

        fn new_before(v: i32, before: *mut Node) -> Self {
            Self {
                val: v,
                prev: null_mut(),
                next: before,
            }
        }
    }
    
    struct List {
        head: *mut Node,
        tail: *mut Node,
        temp: *mut Node,
    }
    
    impl List {
        fn new() -> Self {
            Self {
                head: null_mut(),
                tail: null_mut(),
                temp: null_mut(),
            }
        }
        
        fn push(&mut self, val: i32) {
            unsafe {
                if self.head.is_null() {
                    self.head = Box::into_raw(Box::new(Node::new(val)));
                    self.tail = self.head;
                } else {
                    let tmp_ptr = self.tail;
                    self.tail = Box::into_raw(Box::new(Node::new_after(val, tmp_ptr)));
                    (*tmp_ptr).next = self.tail;
                }
            }
        }
        
        fn next(&mut self) -> Option<i32> {
            unsafe {
                if self.temp.is_null() {
                    self.temp = self.head;
                    return Some((*self.temp).val);
                } else if !(*self.temp).next.is_null() {
                    self.temp = (*self.temp).next;
                    return Some((*self.temp).val);
                } else {
                    self.temp = null_mut();
                    return None;
                }
            }
        }
        
        fn _self_drop(&mut self) {
            unsafe {
                let mut tmp_0 = self.head;
                while !(*tmp_0).next.is_null() {
                    let tmp = (*tmp_0).next;
                    // println!("delete {}.", (*tmp_0).val);
                    dealloc(tmp_0 as *mut u8, Layout::new::<Node>());
                    tmp_0 = tmp;
                }
                // println!("delete {}.", (*tmp_0).val);
                dealloc(tmp_0 as *mut u8, Layout::new::<Node>());
            }
        }
    }
    
    impl Drop for List {
        fn drop(&mut self) {
            unsafe {
                let mut tmp_0 = self.head;
                while !(*tmp_0).next.is_null() {
                    let tmp = (*tmp_0).next;
                    // println!("delete {}.", (*tmp_0).val);
                    dealloc(tmp_0 as *mut u8, Layout::new::<Node>());
                    tmp_0 = tmp;
                }
                // println!("delete {}.", (*tmp_0).val);
                dealloc(tmp_0 as *mut u8, Layout::new::<Node>());
            }
        }
    }
    
    #[test]
    fn test_unsafe_fn() {
        let mut list = List::new();
        list.push(0);
        list.push(1);
        list.push(2);
        list.push(3);
        unsafe {
            let tmp = (*list.head).next;
            println!("{}", (*tmp).val);
            list._self_drop();
            println!("{}", (*tmp).val);
        }
    }
}