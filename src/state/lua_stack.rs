use crate::api::{consts::LUA_REGISTRYINDEX, lua_state::LuaUpValueIndex};

use super::{closure::Closure, lua_value::LuaValue};
use std::{collections::HashMap, rc::Rc, cell::RefCell};

pub struct LuaStack {
    pub slots: Vec<LuaValue>,
    pub top: i32,
    pub closure: Rc<Closure>,
    pub varargs: Vec<LuaValue>,
    pub pc: i32,
    pub registry: LuaValue,
    pub openuvs: HashMap<i32, LuaValue>,        // local register, upvalues
}

impl LuaStack {
    pub fn new(size: usize, closure: Rc<Closure>, registry: LuaValue) -> Self {
        let mut slots = Vec::<LuaValue>::with_capacity(size);
        for _ in 0..size {
            slots.push(LuaValue::Nil);
        }

        Self {
            slots: slots,
            top: 0,
            closure: closure,
            varargs: vec![],
            pc: 0,
            registry: registry,
            openuvs: HashMap::new(),
        }
    }

    pub fn check(&mut self, n: i32) {
        let free = self.slots.len() as i32 - self.top;
        for _ in free..n {
            self.slots.push(LuaValue::Nil);
        }
    }

    pub fn push(&mut self, val: LuaValue) {
        if self.top as usize == self.slots.len() {
            panic!("stack overflow!");
        }
        self.slots[self.top as usize] = val;
        self.top += 1;
    }

    pub fn pop(&mut self) -> LuaValue {
        if self.top < 1 {
            panic!("stack overflow!");
        }
        self.top -= 1;
        let val = self.slots[self.top as usize].clone();
        self.slots[self.top as usize] = LuaValue::Nil;
        val
    }

    pub fn absIndex(&self, idx: i32) -> i32 {
        if idx >= 0 || idx <= LUA_REGISTRYINDEX as i32 {
            return idx;
        }
        idx + self.top + 1
    }

    pub fn isValid(&self, idx: i32) -> bool {
        if idx < LUA_REGISTRYINDEX as i32 {        // upvalues
            let uvIdx = LUA_REGISTRYINDEX as i32 - idx - 1;
            let c = &self.closure;
            return (!c.is_fake) && uvIdx < c.upvals.borrow().len() as i32;
        }
        if idx == LUA_REGISTRYINDEX as i32 {
            return true;
        }
        let absIdx = self.absIndex(idx);
        (absIdx > 0) && (absIdx <= self.top)
    }

    pub fn get(&self, idx: i32) -> LuaValue {
        if idx < LUA_REGISTRYINDEX as i32 {    // upvalues
            let uvIdx = LUA_REGISTRYINDEX as i32- idx - 1;
            let c = &self.closure;
            if c.is_fake || uvIdx >= c.upvals.borrow().len() as i32 {
                return LuaValue::Nil;
            }
            return c.upvals.borrow().get(uvIdx as usize).unwrap().clone();
        }

        if idx == LUA_REGISTRYINDEX as i32 {
            return self.registry.clone();
        }

        let absIdx = self.absIndex(idx);
        if (absIdx > 0) && (absIdx <= self.top) {
            return self.slots[absIdx as usize - 1].clone();
        }
        LuaValue::Nil
    }

    pub fn set(&mut self, idx: i32, val: LuaValue) {
        if idx < LUA_REGISTRYINDEX as i32 {    // upvalues
            let uvIdx = LUA_REGISTRYINDEX as i32- idx - 1;
            let c = Rc::clone(&self.closure);
            let _is_fake = c.is_fake;
            let _up_len = c.upvals.borrow().len() as i32;
            if (!_is_fake) || uvIdx < _up_len {
                if let Some(r_val) = c.upvals.borrow_mut().get_mut(uvIdx as usize) {
                    *r_val = val;
                }
            }
            return;
        }

        if idx == LUA_REGISTRYINDEX as i32 {
            self.registry = val;
            return;
        }

        let absIdx = self.absIndex(idx);
        if (absIdx > 0) && (absIdx <= self.top) {
            self.slots[absIdx as usize - 1] = val;
            return;
        }
        panic!("invalid index!");
    }

    pub fn reverse(&mut self, mut from: i32, mut to: i32) {
        while from < to {
            self.slots.swap(from as usize, to as usize);
            from += 1;
            to -= 1;
        }
    }

    pub fn popN(&mut self, n: i32) -> Vec<LuaValue> {
        let mut vals = Vec::<LuaValue>::with_capacity(n as usize);
        for _ in 0..n {
            vals.push(self.pop());
        }
        vals.reverse();
        vals
    }

    pub fn pushN(&mut self, vals: Vec<LuaValue>, mut n: i32) {
        let nVals = vals.len() as i32;
        if n < 0 {
            n = nVals;
        }
        for i in 0..n {
            if i < nVals {
                self.push(vals[i as usize].clone());
            } else {
                self.push(LuaValue::Nil);
            }
        }
    }

    pub fn SetTop(&mut self, idx: i32) {
        let new_top = self.absIndex(idx);
        if new_top < 0 {
            panic!("stack underflow!");
        }

        let n = self.top - new_top;
        if n > 0 {
            for _ in 0..n {
                self.pop();
            }
        } else if n < 0 {
            for _ in n..0 {
                self.push(LuaValue::Nil);
            }
        }
    }
}