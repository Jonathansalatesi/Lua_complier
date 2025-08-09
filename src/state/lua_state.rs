use std::{cell::RefCell, rc::Rc, result};

use crate::{api::{consts::*, lua_state::LuaAPI, lua_vm::LuaVM}, binchunk::{self, binary_chunk::Prototype}, state::{api_arith::OPERATORS, lua_value::{callMetamethod, getMetafield}}, vm::{inst_operators::le, instruction::Instruction, opcodes::{OPCODES, OP_RETURN}}};
use crate::binchunk::binary_chunk::LUA_SIGNATURE;
use crate::compiler::codegen::compile;
use super::{api_arith, api_compare::{self, eq}, closure::Closure, lua_stack::LuaStack, lua_table::{newLuaTable, newTable, LuaTable}, lua_value::{getMetatable, setMetatable, LuaValue}};

pub struct LuaState {
    pub registry: LuaValue,
    frames: Vec<LuaStack>,
}

impl LuaState {
    pub fn new() -> Self {
        let fake_proto = Rc::new(Prototype::FakeProto());
        let fake_closure = Rc::new(Closure::new(fake_proto));

        let registry = newTable(0, 0);
        registry.borrow_mut().Put(LuaValue::Integer(LUA_RIDX_GLOBALS), newLuaTable(0, 0));
        let fake_frame = LuaStack::new(20, fake_closure, LuaValue::Table(Rc::clone(&registry)));
        LuaState {
            registry: LuaValue::Table(registry),
            frames: vec![fake_frame],
        }
    }

    pub fn printStack(&self, opname: &'static str) {
        print!("  {} ", opname);
        let top = self.GetTop();
        for i in 1..=top {
            let t = self.Type(i);
            match t {
                LUA_TBOOLEAN => {
                    print!("[{}]", self.ToBoolean(i));
                },
                LUA_TNUMBER => {
                    print!("[{}]", self.ToNumber(i));
                },
                LUA_TSTRING => {
                    print!("[\"{}\"]", self.ToString(i));
                },
                _ => {
                    print!("[{}]", self.TypeName(t));
                }
            }
        }
        println!();
    }

    pub fn stack_mut(&mut self) -> &mut LuaStack {
        self.frames.last_mut().unwrap()
    }

    pub fn stack(&self) -> &LuaStack {
        self.frames.last().unwrap()
    }

    pub fn pushFrame(&mut self, frame: LuaStack) {
        self.frames.push(frame);
    }

    pub fn popFrame(&mut self) -> LuaStack {
        self.frames.pop().unwrap()
    }
}

impl LuaAPI for LuaState {
    fn GetTop(&self) -> i32 {
        self.stack().top
    }

    fn AbsIndex(&self, idx: i32) -> i32 {
        self.stack().absIndex(idx)
    }

    fn CheckStack(&mut self, n: i32) -> bool {
        self.stack_mut().check(n);
        true
    }

    fn pop(&mut self, n: i32) {
        for _ in 0..n {
            let _ = self.stack_mut().pop();
        }
    }

    fn copy(&mut self, from_idx: i32, to_idx: i32) {
        let val = self.stack().get(from_idx);
        self.stack_mut().set(to_idx, val);
    }

    fn PushValue(&mut self, idx: i32) {
        let val = self.stack().get(idx);
        self.stack_mut().push(val);
    }

    fn Replace(&mut self, idx: i32) {
        let val = self.stack_mut().pop();
        self.stack_mut().set(idx, val);
    }

    fn Insert(&mut self, idx: i32) {
        self.Rotate(idx, 1);
    }

    fn Remove(&mut self, idx: i32) {
        self.Rotate(idx, -1);
        self.pop(1);
    }

    fn Rotate(&mut self, idx: i32, n: i32) {
        let t = self.stack().top - 1;
        let p = self.stack().absIndex(idx) - 1;
        let mut m: i32;
        if n >= 0 {
            m = t - n;
        } else {
            m = p - n - 1;
        }
        self.stack_mut().reverse(p, m);
        self.stack_mut().reverse(m + 1, t);
        self.stack_mut().reverse(p, t);
    }

    fn SetTop(&mut self, idx: i32) {
        let newTop = self.stack().absIndex(idx);
        if newTop < 0 {
            panic!("stack underflow!");
        }
        let n = self.stack().top - newTop;
        if n > 0 {
            for _ in 0..n {
                self.stack_mut().pop();
            }
        } else if n < 0 {
            for _ in n..0 {
                self.stack_mut().push(super::lua_value::LuaValue::Nil);
            }
        }
    }

    // api_push.go
    fn PushNil(&mut self) {
        self.stack_mut().push(super::lua_value::LuaValue::Nil);
    }

    fn PushBoolean(&mut self, b: bool) {
        self.stack_mut().push(super::lua_value::LuaValue::Bool(b));
    }

    fn PushInteger(&mut self, n: i64) {
        self.stack_mut().push(super::lua_value::LuaValue::Integer(n));
    }

    fn PushNumber(&mut self, n: f64) {
        self.stack_mut().push(super::lua_value::LuaValue::Number(n));
    }

    fn PushString(&mut self, s: String) {
        self.stack_mut().push(super::lua_value::LuaValue::Str(s));
    }

    // access information from stack
    fn TypeName(&self, tp: i8) -> &'static str {
        match tp {
            LUA_TNONE => "no value",
            LUA_TNIL => "nil",
            LUA_TBOOLEAN => "boolean",
            LUA_TNUMBER => "number",
            LUA_TSTRING => "string",
            LUA_TTABLE => "table",
            LUA_TFUNCTION => "function",
            LUA_TTHREAD => "thread",
            _ => "userdata",
        }
    }

    fn Type(&self, idx: i32) -> i8 {
        if self.stack().isValid(idx) {
            let val = self.stack().get(idx);
            return val.typeOf();
        }
        LUA_TNONE
    }

    fn IsNone(&self, idx: i32) -> bool {
        self.Type(idx) == LUA_TNONE
    }
    
    fn IsNil(&self, idx: i32) -> bool {
        self.Type(idx) == LUA_TNIL
    }

    fn IsNoneOrNil(&self, idx: i32) -> bool {
        self.Type(idx) <= LUA_TNIL
    }

    fn IsBoolean(&self, idx: i32) -> bool {
        self.Type(idx) == LUA_TBOOLEAN
    }

    fn IsString(&self, idx: i32) -> bool {
        let t = self.Type(idx);
        (t == LUA_TSTRING) || (t == LUA_TNUMBER)
    }

    fn IsNumber(&self, idx: i32) -> bool {
        self.ToNumberX(idx).is_some()
    }

    fn IsInteger(&self, idx: i32) -> bool {
        match self.stack().get(idx) {
            LuaValue::Integer(_) => true,
            _ => false,
        }
    }

    fn IsTable(&self, idx: i32) -> bool {
        self.Type(idx) == LUA_TTABLE
    }

    fn IsFunction(&self, idx: i32) -> bool {
        self.Type(idx) == LUA_TFUNCTION
    }

    fn IsThread(&self, idx: i32) -> bool {
        self.Type(idx) == LUA_TTHREAD
    }

    fn ToBoolean(&self, idx: i32) -> bool {
        self.stack().get(idx).ToBoolean()
    }

    fn ToNumber(&self, idx: i32) -> f64 {
        self.ToNumberX(idx).unwrap()
    }

    fn ToNumberX(&self, idx: i32) -> Option<f64> {
        self.stack().get(idx).ToFloat()
    }

    fn ToInteger(&self, idx: i32) -> i64 {
        self.ToIntegerX(idx).unwrap()
    }

    fn ToIntegerX(&self, idx: i32) -> Option<i64> {
        self.stack().get(idx).ToInteger()
    }

    fn ToString(&self, idx: i32) -> String {
        self.ToStringX(idx).unwrap()
    }

    fn ToStringX(&self, idx: i32) -> Option<String> {
        match self.stack().get(idx) {
            LuaValue::Str(s) => Some(s),
            LuaValue::Number(n) => Some(n.to_string()),
            LuaValue::Integer(i) => Some(i.to_string()),
            LuaValue::Bool(b) => Some(b.to_string()),
            _ => None,
        }
    }

    fn ArithOp(&mut self, op: u8) {
        let b: LuaValue;
        let a: LuaValue;
        if op != LUA_OPUNM && op != LUA_OPBNOT {
            b = self.stack_mut().pop();
            a = self.stack_mut().pop();
            if let Some(val) = api_arith::arith(&a, &b, op) {
                self.stack_mut().push(val);
                return;
            }
        } else {
            a = self.stack_mut().pop();
            b = a.clone();
            if let Some(val) = api_arith::arith(&a, &a, op) {
                self.stack_mut().push(val);
                return;
            }
        }
        let _mm_ = OPERATORS[op as usize].0;
        if let Some(res) = callMetamethod(a, b, _mm_, self) {
            self.stack_mut().push(res);
            return;
        }

        panic!("arithmetic error!");
    }

    fn Compare(&mut self, idx1: i32, idx2: i32, op: u8) -> bool {
        if !self.stack().isValid(idx1) || !self.stack().isValid(idx2) {
            return false;
        } else {
            let a = self.stack().get(idx1);
            let b = self.stack().get(idx2);
            if let Some(res) = api_compare::compare_meta(&a, &b, op, self) {
                return res;
            }
            if let Some(res) = api_compare::compare(&a, &b, op) {
                return res;
            }
            panic!("comparison error!");
        }
    }

    fn Len(&mut self, idx: i32) {
        if !self.stack().isValid(idx) {
            return;
        }
        let val = self.stack().get(idx);
        if let Some(res) = callMetamethod(val.clone(), val.clone(), "__len", self) {
            self.stack_mut().push(res);
            return;
        }
        match val {
            LuaValue::Str(s) => {
                self.stack_mut().push(LuaValue::Integer(s.len() as i64));
            },
            LuaValue::Table(t) => {
                self.stack_mut().push(LuaValue::Integer(t.borrow().Len() as i64));
            }
            _ => {
                panic!("length error!");
            }
        }
    }

    fn Concat(&mut self, n: i32) {
        if n == 0 {
            self.stack_mut().push(LuaValue::Str(String::from("")));
        } else if n >= 2 {
            for _ in 1..n {
                if self.IsString(-1) && self.IsString(-2) {
                    let s2 = self.ToString(-1);
                    let mut s1 = self.ToString(-2);
                    let _ = self.stack_mut().pop();
                    let _ = self.stack_mut().pop();
                    s1.push_str(&s2);
                    self.stack_mut().push(LuaValue::Str(s1));
                } else {
                    let b = self.stack_mut().pop();
                    let a = self.stack_mut().pop();
                    let _res_ = callMetamethod(a, b, "__concat", self);
                    if let Some(res) = _res_ {
                        self.stack_mut().push(res);
                        continue;
                    }
                    panic!("concatenation error!");
                }
            }
        }
    }

    fn NewTable(&mut self) {
        self.CreateTable(0, 0);
    }

    fn CreateTable(&mut self, nArr: i32, nRec: i32) {
        let t = LuaTable::new(nArr, nRec);
        self.stack_mut().push(LuaValue::Table(Rc::new(RefCell::new(t))));
    }

    fn GetTable(&mut self, idx: i32) -> i8 {
        let t = self.stack().get(idx);
        let k = self.stack_mut().pop();
        self.getTable(&t, &k, false)
    }

    fn GetField(&mut self, idx: i32, k: &'static str) -> i8 {
        let t = self.stack().get(idx);
        self.getTable(&t, &LuaValue::Str(String::from(k)), false)
    }

    fn GetI(&mut self, idx: i32, i: i64) -> i8 {
        let t = self.stack().get(idx);
        self.getTable(&t, &LuaValue::Integer(i), false)
    }

    fn SetTable(&mut self, idx: i32) {
        let t = self.stack().get(idx);
        let v = self.stack_mut().pop();
        let k = self.stack_mut().pop();
        self.setTable(&t, &k, &v, false);
    }

    fn SetField(&mut self, idx: i32, k: &'static str) {
        let t = self.stack().get(idx);
        let v = self.stack_mut().pop();
        self.setTable(&t, &LuaValue::Str(String::from(k)), &v, false);     // warning.
    }

    fn SetI(&mut self, idx: i32, n: i64) {
        let t = self.stack().get(idx);
        let v = self.stack_mut().pop();
        self.setTable(&t, &LuaValue::Integer(n), &v, false);
    }

    fn Load(&mut self, chunk: Vec<u8>, chunk_name: &str, mode: &str) -> i32 {
        let proto: Prototype = if is_binary_chunk(&chunk) {
            binchunk::undump(chunk)
        } else {
            let s_chunk: String = chunk.iter().map(|s|{*s as char}).collect();
            compile(s_chunk, chunk_name.to_owned())
        };
        
        let r_len = proto.upvalues.len();
        let mut c = Closure::new(Rc::new(proto));
        if r_len > 0 {       // set _ENV
            let _env_ = &self.registry;
            if let LuaValue::Table(tbl) = _env_ {
                let __env__ = tbl.borrow().Get(&LuaValue::Integer(LUA_RIDX_GLOBALS));
                if let Some(r_val) = c.upvals.borrow_mut().get_mut(0) {
                    *r_val = __env__;
                }
            }
        }
        self.stack_mut().push(LuaValue::Function(Rc::new(c)));
        0
    }

    fn Call(&mut self, mut nArgs: i32, nResults: i32) {
        let mut val = self.stack().get(-(nArgs + 1));
        if let LuaValue::Function(_) = val {} else {
            let _mf_ = getMetafield(val.clone(), "__call", self);
            if let LuaValue::Function(_) = _mf_ {
                self.stack_mut().push(val.clone());
                self.Insert(-(nArgs + 2));
                nArgs += 1;
                val = _mf_;
            }
        }

        if let LuaValue::Function(c) = val {
            match c.rustFunc {
                None => self.callLuaClosure(nArgs, nResults, Rc::clone(&c)),
                Some(_) => self.callRustClosure(nArgs, nResults, Rc::clone(&c)),
            };
        } else {
            panic!("not function!");
        }
    }

    fn PushRustFunction(&mut self, f: crate::api::lua_state::RustFn) {
        self.stack_mut().push(LuaValue::newRustClosure(f, 0));
    }

    fn IsRustFunction(&self, idx: i32) -> bool {
        let val = self.stack().get(idx);
        if let LuaValue::Function(f) = val {
            if let Some(_) = f.rustFunc {
                return true;
            }
        }
        false
    }

    fn ToGoFunction(&self, idx: i32) -> Option<crate::api::lua_state::RustFn> {
        let val = self.stack().get(idx);
        if let LuaValue::Function(f) = val {
            return f.rustFunc;
        }
        None
    }

    fn PushGlobalTable(&mut self) {
        let _global_ = &self.registry;
        if let LuaValue::Table(tbl) = _global_ {
            let __global__ = tbl.borrow().Get(&LuaValue::Integer(LUA_RIDX_GLOBALS));
            self.stack_mut().push(__global__);
        }
    }

    fn GetGlobal(&mut self, name: &'static str) -> i8 {
        let rf_t = &self.registry;
        if let LuaValue::Table(tbl) = rf_t {
            let t = tbl.borrow().Get(&LuaValue::Integer(LUA_RIDX_GLOBALS));
            return self.getTable(&t, &LuaValue::Str(String::from(name)), false);
        }
        -1
    }

    fn SetGlobal(&mut self, name: &'static str) {
        let rf_t = &self.registry;
        if let LuaValue::Table(tbl) = rf_t {
            let t = tbl.borrow().Get(&LuaValue::Integer(LUA_RIDX_GLOBALS));
            let v = self.stack_mut().pop();
            self.setTable(&t, &LuaValue::Str(String::from(name)), &v, false);
        }
    }

    fn Register(&mut self, name: &'static str, f: crate::api::lua_state::RustFn) {
        self.PushRustFunction(f);
        self.SetGlobal(name);
    }

    fn PushGoClosure(&mut self, f: crate::api::lua_state::RustFn, n: i32) {
        let mut _closure_ = Closure::newRustClosure(f, n);
        let closure = &mut _closure_;
        for _ in (1..=n).rev() {
            let val = self.stack_mut().pop();
            if let Some(r_val) = closure.upvals.borrow_mut().get_mut(n as usize - 1) {
                *r_val = val;
            }
        }
        self.stack_mut().push(LuaValue::Function(Rc::new(_closure_)));
    }

    fn GetMetatable(&mut self, idx: i32) -> bool {
        let val = self.stack().get(idx);
        let _mt_ = getMetatable(val, self);
        if !_mt_.IsNil() {
            self.stack_mut().push(_mt_);
            return true;
        } else {
            return false;
        }
    }

    fn SetMetatable(&mut self, idx: i32) {
        let val = self.stack().get(idx);
        let mtVal = self.stack_mut().pop();
        match mtVal {
            LuaValue::Nil => {
                setMetatable(val, None, self);
            },
            LuaValue::Table(tbl) => {
                setMetatable(val, Some(tbl), self);
            },
            _ => {
                panic!("table expected!");
            },
        }
    }

    fn RawSet(&mut self, idx: i32) {
        let t  = self.stack().get(idx);
        let v = self.stack_mut().pop();
        let k = self.stack_mut().pop();
        self.setTable(&t, &k, &v, true);
    }

    fn RawSetI(&mut self, idx: i32, i: i64) {
        let t = self.stack().get(idx);
        let v = self.stack_mut().pop();
        self.setTable(&t, &LuaValue::Integer(i), &v, true);
    }

    fn RawGet(&mut self, idx: i32) -> i8 {
        let t  = self.stack().get(idx);
        let k = self.stack_mut().pop();
        self.getTable(&t, &k, true)
    }

    fn RawGetI(&mut self, idx: i32, i: i64) -> i8 {
        let t  = self.stack().get(idx);
        self.getTable(&t, &LuaValue::Integer(i), true)
    }

    fn RawEqual(&self, idx1: i32, idx2: i32) -> bool {
        if !self.stack().isValid(idx1) || !self.stack().isValid(idx2) {
            return false;
        }
        let a = self.stack().get(idx1);
        let b = self.stack().get(idx2);
        eq(&a, &b)
    }

    fn RawLen(&self, idx: i32) -> u32 {
        let val = self.stack().get(idx);
        match val {
            LuaValue::Str(s) => {
                s.len() as u32
            },
            LuaValue::Table(tbl) => {
                tbl.borrow().Len() as u32
            }
            _ => 0,
        }
    }

    fn Next(&mut self, idx: i32) -> bool {
        let val = self.stack().get(idx);
        if let LuaValue::Table(t) = val {
            let key = self.stack_mut().pop();
            let next_key = t.borrow_mut().nextKey(&key);
            if !next_key.IsNil() {
                self.stack_mut().push(next_key.clone());
                self.stack_mut().push(t.borrow().Get(&next_key));
                return true;
            }
            return false;
        }
        panic!("table expected!");
    }
}

impl LuaState {
    fn getTable(&mut self, t: &LuaValue, k: &LuaValue, raw: bool) -> i8 {
        if let LuaValue::Table(tbl) = t {
            let v = tbl.borrow().Get(k);

            if raw || !v.IsNil() || !tbl.borrow().hasMetafield("__index") {
                self.stack_mut().push(v.clone());
                return v.typeOf();
            }
        } else {
            print!("{t:?}");
        }

        if !raw {
            let mf = getMetafield(t.clone(), "__index", self);
            match mf {
                LuaValue::Table(_) => {
                    return self.getTable(&mf, k, false);
                },
                LuaValue::Function(_) => {
                    self.stack_mut().push(mf);
                    self.stack_mut().push(t.clone());
                    self.stack_mut().push(k.clone());
                    self.Call(2, 1);
                    let v = self.stack().get(-1);
                    return v.typeOf();
                },
                _ => {},
            };
        }
        panic!("index error!");
    }

    fn setTable(&mut self, t: &LuaValue, k: &LuaValue, v: &LuaValue, raw: bool) {
        if let LuaValue::Table(tbl) = t {
            if raw || !tbl.borrow().Get(k).IsNil() || !tbl.borrow().hasMetafield("__newindex") {
                tbl.borrow_mut().Put(k.clone(), v.clone());
                return;
            }
        }

        if !raw {
            let mf = getMetafield(t.clone(), "__newindex", self);
            match mf {
                LuaValue::Table(_) => {
                    self.setTable(&mf, k, v, false);
                    return;
                },
                LuaValue::Function(_) => {
                    self.stack_mut().push(mf);
                    self.stack_mut().push(t.clone());
                    self.stack_mut().push(k.clone());
                    self.stack_mut().push(v.clone());
                    self.Call(3, 0);
                    return;
                },
                _ => {},
            };
        }

        panic!("not a table!");
    }

    fn callLuaClosure(&mut self, nArgs: i32, nResults: i32, c: Rc<Closure>) {
        let nRegs = c.proto.maxStackSize as i32;
        let nParams = c.proto.numParams as i32;
        let isVararg = c.proto.isVararg == 1;

        let mut newStack = LuaStack::new(nRegs as usize + 20, Rc::clone(&c), self.registry.clone());
        // pass args, pop func
        let mut args = self.stack_mut().popN(nArgs);
        self.stack_mut().pop(); // pop func
        if nArgs > nParams {
            // varargs
            for _ in nParams..nArgs {
                newStack.varargs.push(args.pop().unwrap());
            }
            if isVararg {
                newStack.varargs.reverse();
            } else {
                newStack.varargs.clear();
            }
        }
        newStack.pushN(args, nParams as i32);
        newStack.SetTop(nRegs as i32);

        // run closure
        self.pushFrame(newStack);
        self.runLuaClosure();
        newStack = self.popFrame();

        // return results
        if nResults != 0 {
            let nrets = newStack.top - nRegs;
            let results = newStack.popN(nrets);
            self.stack_mut().check(nrets);
            self.stack_mut().pushN(results, nResults);
        }
    }

    fn runLuaClosure(&mut self) {
        loop {
            let mut inst = Instruction::new(self.Fetch());
            inst.Execute(self);
            if inst.Opcode() == OP_RETURN as i32{
                break;
            }
        }
    }

    fn callRustClosure(&mut self, nArgs: i32, nResults: i32, c: Rc<Closure>) {
        let mut newStack = LuaStack::new(nArgs as usize + 20, Rc::clone(&c), self.registry.clone());
        let args = self.stack_mut().popN(nArgs);
        newStack.pushN(args, nArgs);
        let _ = self.stack_mut().pop();
        let rustFunction = c.rustFunc.unwrap();

        self.pushFrame(newStack);
        let r = rustFunction(self);
        newStack = self.popFrame();

        if nResults != 0 {
            let results = newStack.popN(r);
            self.stack_mut().check(results.len() as i32);
            self.stack_mut().pushN(results, nResults);
        }
    }
}

impl LuaVM for LuaState {
    fn PC(&self) -> i32 {
        self.stack().pc
    }

    fn AddPC(&mut self, n: i32) {
        self.stack_mut().pc += n
    }

    fn Fetch(&mut self) -> u32 {
        // let tmp_pc = self.stack().pc;
        // let total_len = self.stack().closure.proto.code.len();
        // if tmp_pc < total_len as i32 {
            let i = self.stack().closure.proto.code[self.stack().pc as usize];
            self.stack_mut().pc += 1;
            i
        // } else {
        //     let i = self.stack().closure.proto.code[total_len - 1];
        //     // self.stack_mut().pc += 1;
        //     i
        // }
    }

    fn GetConst(&mut self, idx: i32) {
        let c = self.stack().closure.proto.constants[idx as usize].clone();
        self.stack_mut().push(c);
    }

    fn GetRK(&mut self, rk: i32) {
        if rk > 0xff {      // constant
            self.GetConst(rk & 0xff);
        } else {            // register
            self.PushValue(rk + 1);
        }
    }

    fn RegisterCount(&self) -> i32 {
        self.stack().closure.proto.maxStackSize as i32
    }

    fn LoadVararg(&mut self, mut n: i32) {
        if n < 0 {
            n = self.stack().varargs.len() as i32;
        }
        self.stack_mut().check(n);
        let v_tmp = self.stack().varargs.clone();
        self.stack_mut().pushN(v_tmp, n);
    }

    fn LoadProto(&mut self, idx: i32) {
        let subProto = self.stack_mut().closure.proto.protos[idx as usize].clone();
        let mut _closure_ = Closure::new(Rc::new(subProto));
        let closure = &mut _closure_;
        for i in 0..closure.proto.upvalues.len() {
            let uvInfo = &closure.proto.upvalues[i];
            let uvIdx = uvInfo.idx as i32;
            if uvInfo.instack == 1 {
                let _openuv_ = self.stack().openuvs.get(&uvIdx);
                match _openuv_ {
                    Some(openuv) => {
                        if let Some(r_val) = closure.upvals.borrow_mut().get_mut(i) {
                            *r_val = openuv.clone();
                        }
                    },
                    None => {
                        if let Some(r_val) = closure.upvals.borrow_mut().get_mut(i) {
                            *r_val = self.stack().slots[uvIdx as usize].clone();
                        }
                        self.stack_mut().openuvs.insert(uvIdx, closure.upvals.borrow().get(i).unwrap().clone());
                    }
                }
            } else {
                if let Some(r_val) = closure.upvals.borrow_mut().get_mut(i) {
                    *r_val = self.stack().closure.upvals.borrow().get(uvIdx as usize).unwrap().clone();
                }
            }
        }
        self.stack_mut().push(LuaValue::Function(Rc::new(_closure_)));
    }

    fn CloseUpvalues(&mut self, a: i32) {
        let mut to_del = vec![];
        for (k, _) in &self.stack().openuvs {
            if *k >= a - 1 {
                to_del.push(*k);
            }
        }
        for k in to_del {
            let _ = self.stack_mut().openuvs.remove(&k);
        }
    }
}

pub fn is_binary_chunk(data: &Vec<u8>) -> bool {
    if data.len() > 4 {
        if data[..4] == LUA_SIGNATURE {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let a = [1, 2, 4];
        let b = [1, 2, 5];
        assert_eq!(a == b, false);
    }
}