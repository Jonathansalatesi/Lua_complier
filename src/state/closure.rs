use std::{fmt::Debug, hash::{Hash, Hasher}, rc::Rc, cell::RefCell};
use crate::{binchunk::binary_chunk::Prototype, number::math::random, api::lua_state::RustFn};

use super::lua_value::LuaValue;

#[derive(Debug)]
pub struct Closure {
    pub proto: Rc<Prototype>,
    pub rustFunc: Option<RustFn>,
    pub upvals: RefCell<Vec<LuaValue>>,
    pub is_fake: bool,
    rdm: usize,
}

impl Hash for Closure {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rdm.hash(state);
    }
}

impl Closure {
    pub fn new(proto: Rc<Prototype>) -> Self {
        let n_up_vals = proto.upvalues.len();
        let mut upvals = Vec::with_capacity(n_up_vals);
        if n_up_vals > 0 {
            for _ in 0..n_up_vals {
                upvals.push(LuaValue::Nil);
            }
        }
        Self {
            proto: proto,
            rustFunc: None,
            upvals: RefCell::new(upvals),
            is_fake: false,
            rdm: random(),
        }
    }

    pub fn newRustClosure(f: RustFn, n_up_vals: i32) -> Self {
        let mut upvals = Vec::with_capacity(n_up_vals as usize);
        if n_up_vals > 0 {
            for _ in 0..n_up_vals {
                upvals.push(LuaValue::Nil);
            }
        }
        Self {
            proto: Rc::new(Prototype::FakeProto()),
            rustFunc: Some(f),
            upvals: RefCell::new(upvals),
            is_fake: false,
            rdm: random(),
        }
    }

    pub fn newFakeClosure() -> Self {
        Self {
            proto: Rc::new(Prototype::FakeProto()),
            rustFunc: None,
            upvals: RefCell::new(vec![]),
            is_fake: true,
            rdm: 0,
        }
    }
}

