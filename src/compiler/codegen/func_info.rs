use std::{
    collections::HashMap,
    ptr::null_mut
};
use super::super::{
    ast::exp::Exp,
    lexer::token::*,
};
use crate::{
    vm::{
        instruction::*,
        opcodes::*,
        fpb::*
    }, 
    state::lua_value::*
};

pub fn arith_to_bitwise_binops(op: Token) -> Option<u8> {
    match op {
        TOKEN_OP_ADD => Some(OP_ADD),
        TOKEN_OP_SUB => Some(OP_SUB),
        TOKEN_OP_MUL => Some(OP_MUL),
        TOKEN_OP_MOD => Some(OP_MOD),
        TOKEN_OP_POW => Some(OP_POW),
        TOKEN_OP_DIV => Some(OP_DIV),
        TOKEN_OP_IDIV => Some(OP_IDIV),
        TOKEN_OP_BAND => Some(OP_BAND),
        TOKEN_OP_BOR => Some(OP_BOR),
        TOKEN_OP_BXOR => Some(OP_BXOR),
        TOKEN_OP_SHL => Some(OP_SHL),
        TOKEN_OP_SHR => Some(OP_SHR),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct Constants{
    pub keys: Vec<LuaValue>,
    pub values: Vec<i32>,
}

impl Constants {
    pub fn new() -> Self {
        Constants {
            keys: Vec::new(),
            values: Vec::new(),
        }
    }
    
    pub fn get(&self, key: &LuaValue) -> Option<i32> {
        for (i, k) in self.keys.iter().enumerate() {
            if k == key {
                return Some(self.values[i]);
            }
        }
        None
    }
    
    pub fn len(&self) -> usize {
        self.keys.len()
    }
    
    pub fn insert(&mut self, key: LuaValue, values: i32) {
        self.keys.push(key);
        self.values.push(values);
    }
}

#[derive(Clone, Debug)]
pub struct FuncInfo {
    pub constants: Constants,
    pub used_regs: i32,
    pub max_regs: i32,
    pub scope_level: i32,
    pub local_vars: Vec<*mut LocalVarInfo>,
    pub local_names: HashMap<String, *mut LocalVarInfo>,
    pub breaks: Vec<Option<Vec<i32>>>,
    pub parent: *mut FuncInfo,
    pub up_values: HashMap<String, UpValInfo>,
    pub insts: Vec<u32>,
    pub sub_funcs: Vec<*mut FuncInfo>,
    pub num_params: i32,
    pub is_vararg: bool,
}

impl FuncInfo {
    pub fn new(fd: &Exp) -> Self {
        if let Exp::FuncDefExp { par_list, is_vararg, .. } = fd {
            return FuncInfo {
                parent: null_mut(),
                sub_funcs: vec![],
                constants: Constants::new(),
                up_values: HashMap::new(),
                local_names: HashMap::new(),
                local_vars: vec![],
                breaks: vec![],
                insts: vec![],
                is_vararg: *is_vararg,
                num_params: par_list.len() as i32,
                used_regs: 0,
                max_regs: 0,
                scope_level: 0,
            };
        }
        panic!("input params type error: not FuncDefExp");
    }
    
    pub fn ref_set_parent(&mut self, parent: *mut FuncInfo) {
        Self::set_parent(self as *mut Self, parent);
    }
    
    pub fn set_parent(tmp: *mut Self, parent: *mut FuncInfo) {
        unsafe {
            (*tmp).parent = parent;
            for sub_ptr in (*parent).sub_funcs.iter() {
                if  *sub_ptr == tmp {
                    return;
                }
            }
            (*parent).sub_funcs.push(tmp);
        }
    }
    
    pub fn new_ptr(parent: *mut FuncInfo, fd: &Exp) -> *mut Self {
        if let Exp::FuncDefExp { par_list, is_vararg, .. } = fd {
            let func_info_ret = Box::into_raw(Box::new(FuncInfo {
                parent,
                sub_funcs: vec![],
                constants: Constants::new(),
                up_values: HashMap::new(),
                local_names: HashMap::new(),
                local_vars: vec![],
                breaks: vec![],
                insts: vec![],
                is_vararg: *is_vararg,
                num_params: par_list.len() as i32,
                used_regs: 0,
                max_regs: 0,
                scope_level: 0,
            }));
            
            if !parent.is_null() {
                unsafe {
                    for sub_ptr in (*parent).sub_funcs.iter() {
                        if  *sub_ptr == func_info_ret{
                            return func_info_ret;
                        }
                    }
                    (*parent).sub_funcs.push(func_info_ret);
                }
            }
            return func_info_ret;
        }
        panic!("input params type error: not FuncDefExp");
    }
    
    fn _self_drop(&mut self) {
        Self::_self_drop_ptr(self as *mut Self);
    }
    
    fn _self_drop_ptr(ptr_self: *mut Self) {
        // pub local_vars: Vec<*mut LocalVarInfo>,
        // pub local_names: HashMap<String, *mut LocalVarInfo>,
        // pub parent: *mut FuncInfo,
        // pub sub_funcs: Vec<*mut FuncInfo>,
        unsafe {
            if (*ptr_self).sub_funcs.len() == 0 {
                let _ = Box::from_raw(ptr_self);
                return;
            } else {
                for ptr_i in (*ptr_self).sub_funcs.iter() {
                    if !ptr_i.is_null() {
                        let mut_self = &mut **ptr_i;
                        mut_self._self_drop();
                    }
                }
                let _ = Box::from_raw(ptr_self);
                return;
            }
        }
    }
    
    pub fn index_of_constant(&mut self, k: &LuaValue) -> i32 {
        if let Some(idx) = self.constants.get(k) {
            return idx;
        }
        let idx = self.constants.len() as i32;
        self.constants.insert(k.clone(), idx);
        idx
    }
    
    pub fn alloc_reg(&mut self) -> i32 {
        self.used_regs += 1;
        if self.used_regs >= 255 {
            panic!("function or expression needs too many registers.");
        }
        if self.used_regs > self.max_regs {
            self.max_regs = self.used_regs;
        }
        self.used_regs - 1
    }
    
    pub fn free_reg(&mut self) {
        self.used_regs -= 1;
    }
    
    pub fn alloc_regs(&mut self, n: i32) -> i32 {
        for _ in 0..n {
            self.alloc_reg();
        }
        self.used_regs - n
    }
    
    pub fn free_regs(&mut self, n: i32) {
        for _ in 0..n {
            self.free_reg();
        }
    }
    
    pub fn enter_scope(&mut self, break_table: bool) {
        self.scope_level += 1;
        if break_table {
            self.breaks.push(Some(vec![]));
        } else {
            self.breaks.push(None);
        }
    }
    
    pub fn add_local_var(&mut self, name: &str) -> i32 {
        let _prev_ = if let Some(val) = self.local_names.get(name) {
            *val
        } else {
            null_mut()
        };
        
        let new_var = Box::into_raw(Box::new(LocalVarInfo {
            name: name.to_owned(),
            prev: _prev_,
            scope_level: self.scope_level,
            slot: self.alloc_reg(),
            captured: false,
        }));
        self.local_vars.push(new_var);
        self.local_names.insert(name.to_owned(), new_var);
        unsafe {
            (*new_var).slot
        }
    }
    
    pub fn slot_of_local_var(&self, name: &str) -> i32 {
        if let Some(local_var) = self.local_names.get(name) {
            if !local_var.is_null() {
                unsafe {
                    return (**local_var).slot;
                }
            }
        }
        -1
    }
    
    pub fn exit_scope(&mut self) {
        let length_breaks = self.breaks.len();
        let pending_break_jmps = if  length_breaks > 0 {
            self.breaks.remove(length_breaks - 1)
        } else {
            None
        };
        // let pending_break_jmps = self.breaks.remove(length_breaks - 1);
        let a = self.get_jmp_argA();
        if let Some(_pending_break_jmps) = pending_break_jmps {
            for pc in _pending_break_jmps {
                let sBx = self.pc() - pc;
                let i = (sBx - MAXARG_sBx) << 14 | a << 6 | OP_JMP as i32;
                self.insts[pc as usize] =  i as u32;
            }
        }
        
        self.scope_level -= 1;
        let self_ptr = self as *mut FuncInfo;
        
        unsafe {
            for (_, local_var) in (&*self_ptr).local_names.iter() {
                if !local_var.is_null() && (**local_var).scope_level > self.scope_level {
                    self.remove_local_var(*local_var);
                }
            }
        }
    }
    
    pub fn remove_local_var(&mut self, local_var: *mut LocalVarInfo) {
        self.free_reg();
        unsafe {
            if (*local_var).prev.is_null() {
                if let Some(res) = self.local_names.remove(&(*local_var).name) {
                    if !res.is_null() {
                        let _ = Box::from_raw(res);
                    }
                }
            } else if (*(*local_var).prev).scope_level == (*local_var).scope_level {
                self.remove_local_var((*local_var).prev);
            } else {
                self.local_names.insert((*local_var).name.clone(), (*local_var).prev);
            }
        }
    }
    
    pub fn add_break_jmp(&mut self, pc: i32) {
        let tmp_scope_level = self.scope_level;
        for i in (0..=tmp_scope_level).rev() {
            if i < self.breaks.len() as i32 {
                if let Some(ref mut vec_tmp) = self.breaks[i as usize] {
                    vec_tmp.push(pc);
                    return;
                }
            }
        }
        panic!("<break> at line ? not inside a loop!");
    }
    
    pub fn index_of_upVal(&mut self, name: &str) -> i32 {
        if let Some(upval) = self.up_values.get(name) {
            return upval.index;
        }
        if !self.parent.is_null() {
            let _parent = self.parent;
            unsafe {
                if let Some(loc_var) = (*_parent).local_names.get(name) {
                    let idx = self.up_values.len();
                    self.up_values.insert(name.to_owned(), UpValInfo {
                        local_var_slot: (**loc_var).slot,
                        up_val_index: -1,
                        index: idx as i32,
                    });
                    (**loc_var).captured = true;
                    return idx as i32;
                }
                let __parent = &mut *_parent as &mut FuncInfo;
                let uv_idx = __parent.index_of_upVal(name);
                if uv_idx >= 0 {
                    let idx = self.up_values.len();
                    self.up_values.insert(name.to_owned(), UpValInfo {
                        local_var_slot: -1,
                        up_val_index: uv_idx,
                        index: idx as i32,
                    });
                    return idx as i32;
                }
            }
        }
        -1
    }
    
    pub fn get_jmp_argA(&mut self) -> i32 {
        let mut has_captured_loc_vars = false;
        let mut min_slot_of_loc_vars = self.max_regs;
        
        for (_, local_var) in self.local_names.iter() {
            unsafe {
                if (**local_var).scope_level == self.scope_level {
                    let mut v = *local_var;
                    while !v.is_null() && (*v).scope_level == self.scope_level {
                        if (*v).captured {
                            has_captured_loc_vars = true;
                        }
                        if (*v).slot < min_slot_of_loc_vars && !(*v).name.starts_with('(') {
                            min_slot_of_loc_vars = (*v).slot;
                        }
                        v = (*v).prev;
                    }
                }
            }
        }
        
        if has_captured_loc_vars {
            min_slot_of_loc_vars + 1
        } else {
            0
        }
    }
    
    pub fn emit_ABC(&mut self, opcode: i32, a: i32, b: i32, c: i32) {
        let i = b << 23 | c << 14 | a << 6 | opcode;
        self.insts.push(i as u32);
    }

    pub fn emit_ABx(&mut self, opcode: i32, a: i32, bx: i32) {
        let i = bx << 14 | a << 6 | opcode;
        self.insts.push(i as u32);
    }

    pub fn emit_AsBx(&mut self, opcode: i32, a: i32, b: i32) {
        let i = (b + MAXARG_sBx) << 14 | a << 6 | opcode;
        self.insts.push(i as u32);
    }

    pub fn emit_Ax(&mut self, opcode: i32, ax: i32) {
        let i = ax << 6 | opcode;
        self.insts.push(i as u32);
    }
    
    pub fn pc(&self) -> i32 {
        self.insts.len() as i32 - 1
    }
    
    pub fn fix_sBx(&mut self, pc: i32, sBx: i32) {
        let i = self.insts[pc as usize];
        let i = i << 18 >> 18;
        let i = i | ((sBx + MAXARG_sBx)  as u32) << 14;
        self.insts[pc as usize] = i;
    }

    pub fn emit_move(&mut self, a: i32, b: i32) {
        self.emit_ABC(OP_MOVE as i32, a, b, 0)
    }

    pub fn emit_load_nil(&mut self, a: i32, n: i32) {
        self.emit_ABC(OP_LOADNIL as i32, a, n - 1, 0)
    }

    pub fn emit_load_bool(&mut self, a: i32, b: i32, c: i32) {
        self.emit_ABC(OP_LOADBOOL as i32, a, b, c);
    }

    pub fn emit_load_K(&mut self, a: i32, k: &LuaValue) {
        let idx = self.index_of_constant(k);
        if idx < (1 << 18) {
            self.emit_ABx(OP_LOADK as i32, a, idx);
        } else {
            self.emit_ABx(OP_LOADKX as i32, a, 0);
            self.emit_Ax(OP_EXTRAARG as i32, idx);
        }
    }

    // r[a], r[a+1], ..., r[a+b-2] = vararg
    pub fn emit_vararg(&mut self, a: i32, n: i32) {
        self.emit_ABC(OP_VARARG as i32, a, n + 1, 0);
    }

    // r[a] = emitClosure(proto[bx])
    pub fn emit_closure(&mut self, a: i32, bx: i32) {
        self.emit_ABx(OP_CLOSURE as i32, a, bx);
    }

    // r[a] = {}
    pub fn emit_new_table(&mut self, a: i32, n_arr: i32, n_rec: i32) {
        self.emit_ABC(OP_NEWTABLE as i32, a, int2fb(n_arr as usize) as i32, int2fb(n_rec as usize) as i32);
    }

    // r[a][(c-1)*FPF+i] := r[a+i], 1 <= i <= b
    pub fn emit_set_list(&mut self, a: i32, b: i32, c: i32) {
        self.emit_ABC(OP_SETLIST as i32, a, b, c);
    }

    // r[a] := r[b][rk(c)]
    pub fn emit_get_table(&mut self, a: i32, b: i32, c: i32) {
        self.emit_ABC(OP_GETTABLE as i32, a, b, c);
    }

    // r[a][rk(b)] = rk(c)
    pub fn emit_set_table(&mut self, a: i32, b: i32, c: i32) {
        self.emit_ABC(OP_SETTABLE as i32, a, b, c);
    }

    // r[a] = upval[b]
    pub fn emit_get_upval(&mut self, a: i32, b: i32) {
        self.emit_ABC(OP_GETUPVAL as i32, a, b, 0);
    }

    // upval[b] = r[a]
    pub fn emit_set_upval(&mut self, a: i32, b: i32) {
        self.emit_ABC(OP_SETUPVAL as i32, a, b, 0);
    }

    // r[a] = upval[b][rk(c)]
    pub fn emit_get_tab_up(&mut self, a: i32, b: i32, c: i32) {
        self.emit_ABC(OP_GETTABUP as i32, a, b, c);
    }

    // upval[a][rk(b)] = rk(c)
    pub fn emit_set_tab_up(&mut self, a: i32, b: i32, c: i32) {
        self.emit_ABC(OP_SETTABUP as i32, a, b, c);
    }

    // r[a], ..., r[a+c-2] = r[a](r[a+1], ..., r[a+b-1])
    pub fn emit_call(&mut self, a: i32, n_args: i32, n_rec: i32) {
        self.emit_ABC(OP_CALL as i32, a, n_args + 1, n_rec + 1);
    }

    // return r[a](r[a+1], ... ,r[a+b-1])
    pub fn emit_tail_call(&mut self, a: i32, n_args: i32) {
        self.emit_ABC(OP_TAILCALL as i32, a, n_args + 1, 0);
    }

    // return r[a], ... ,r[a+b-2]
    pub fn emit_return(&mut self, a: i32, n: i32) {
        self.emit_ABC(OP_RETURN as i32, a, n + 1, 0);
    }

    // r[a+1] := r[b]; r[a] := r[b][rk(c)]
    pub fn emit_self(&mut self, a: i32, b: i32, c: i32) {
        self.emit_ABC(OP_SELF as i32, a, b, c);
    }

    // pc+=sBx; if (a) close all upvalues >= r[a - 1]
    pub fn emit_jmp(&mut self, a: i32, sbx: i32) -> i32 {
        self.emit_AsBx(OP_JMP as i32, a, sbx);
        self.insts.len() as i32 - 1
    }
    
    // if not (r[a] <=> c) then pc++
    pub fn emit_test(&mut self, a: i32, c: i32) {
        self.emit_ABC(OP_TEST as i32, a, 0, c);
    }
    
    // if (r[b] <=> c) then r[a] := r[b] else pc++
    pub fn emit_test_set(&mut self, a: i32, b: i32, c: i32) {
        self.emit_ABC(OP_TESTSET as i32, a, b, c);
    }

    pub fn emit_for_prep(&mut self, a: i32, sbx: i32) -> i32 {
        self.emit_AsBx(OP_FORPREP as i32, a, sbx);
        self.insts.len() as i32 - 1
    }

    pub fn emit_for_loop(&mut self, a: i32, sbx: i32) -> i32 {
        self.emit_AsBx(OP_FORLOOP as i32, a, sbx);
        self.insts.len() as i32 - 1
    }

    pub fn emit_tfor_call(&mut self, a: i32, c: i32) {
        self.emit_ABC(OP_TFORCALL as i32, a, 0, c);
    }

    pub fn emit_tfor_loop(&mut self, a: i32, sbx: i32) {
        self.emit_AsBx(OP_TFORCALL as i32, a, sbx);
    }
    
    // r[a] = op r[b]
    pub fn emit_unary_op(&mut self, op: i32, a: i32, b: i32) {
        match op{
            TOKEN_OP_NOT => {
                self.emit_ABC(OP_NOT as i32, a, b, 0);
            },
            TOKEN_OP_BNOT => {
                self.emit_ABC(OP_BNOT as i32, a, b, 0);
            },
            TOKEN_OP_LEN => {
                self.emit_ABC(OP_LEN as i32, a, b, 0);
            },
            TOKEN_OP_UNM => {
                self.emit_ABC(OP_UNM as i32, a, b, 0);
            },
            _ => {},
        }
    }
    
    // r[a] = rk[b] op rk[c]
    // arith & bitwise & relational
    pub fn emit_binary_op(&mut self, op: i32, a: i32, b: i32, c: i32) {
        if let Some(opcode) = arith_to_bitwise_binops(op) {
            self.emit_ABC(opcode as i32, a, b, c);
        } else {
            match op {
                TOKEN_OP_EQ => self.emit_ABC(OP_EQ as i32, 1, b, c),
                TOKEN_OP_NE => self.emit_ABC(OP_EQ as i32, 0, b, c),
                TOKEN_OP_LT => self.emit_ABC(OP_LT as i32, 1, b, c),
                TOKEN_OP_GT => self.emit_ABC(OP_LT as i32, 1, c, b),
                TOKEN_OP_LE => self.emit_ABC(OP_LE as i32, 1, b, c),
                TOKEN_OP_GE => self.emit_ABC(OP_LE as i32, 1, c, b),
                _ => {},
            }
            self.emit_jmp(0, 1);
            self.emit_load_bool(a, 0, 1);
            self.emit_load_bool(a, 1, 0);
        }
    }
    
    pub fn close_open_upvals(&mut self) {
        let a = self.get_jmp_argA();
        if a > 0 {
            self.emit_jmp(a, 0);
        }
    }
}

// impl Drop for FuncInfo {
//     fn drop(&mut self) {
//         self._self_drop();
//     }
// }

#[derive(Clone, Debug)]
pub struct LocalVarInfo {
    prev: *mut LocalVarInfo,
    name: String,
    scope_level: i32,
    slot: i32,
    captured: bool,
}

#[derive(Clone, Debug)]
pub struct UpValInfo {
    pub local_var_slot: i32,
    pub up_val_index: i32,
    pub index: i32,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    
    #[derive(Debug)]
    pub struct Example {
        a: i32,
        b: Vec<i32>,
        c: HashMap<i32, i32>,
    }
    
    impl Example {
        pub fn new(a: i32) -> Self {
            Self {
                a,
                b: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
                c:  HashMap::new(),
            }
        }
    }

    #[test]
    fn unsafe_ptr() {
        let mut example = Example::new(0);
        let ptr_example = &mut example as *mut Example;
        unsafe {
            for i in &((*ptr_example).b) {
                (*ptr_example).a += i;
            }
        }
        println!("{}", example.a);
        println!("{:?}", example);
        drop(example);
        // println!("{:?}", example);
    }
    
    #[test]
    fn test_hash() {
        let mut map = HashMap::new();
        map.insert(String::from("a"), 1);
        map.insert(String::from("b"), 2);
        map.insert(String::from("c"), 3);
        assert_eq!(map.get("a"), Some(&1));
        assert_eq!(map.get("b"), Some(&2));
    }
}