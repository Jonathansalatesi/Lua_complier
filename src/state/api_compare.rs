use super::{lua_state::LuaState, lua_value::{callMetamethod, LuaValue}};
use crate::api::consts::*;

pub fn compare_meta(a: &LuaValue, b: &LuaValue, op: u8, ls: &mut LuaState) -> Option<bool> {
    match op {
        LUA_OPEQ => {
            match a {
                LuaValue::Table(_) => {
                    match b {
                        LuaValue::Table(_) => {
                            let _res_ = callMetamethod(a.clone(), b.clone(), "__eq", ls);
                            if let Some(res) = _res_ {
                                return Some(res.ToBoolean());
                            } else {
                                return None;
                            }
                        },
                        _ => None,
                    }
                },
                _ => None,
            }
        },
        LUA_OPLT => {
            match a {
                LuaValue::Table(_) => {
                    match b {
                        LuaValue::Table(_) => {
                            let _res_ = callMetamethod(a.clone(), b.clone(), "__lt", ls);
                            if let Some(res) = _res_ {
                                return Some(res.ToBoolean());
                            } else {
                                return None;
                            }
                        },
                        _ => None,
                    }
                },
                _ => None,
            }
        },
        LUA_OPLE => {
            match a {
                LuaValue::Table(_) => {
                    match b {
                        LuaValue::Table(_) => {
                            let _res_ = callMetamethod(a.clone(), b.clone(), "__le", ls);
                            if let Some(res) = _res_ {
                                return Some(res.ToBoolean());
                            } else {
                                return None;
                            }
                        },
                        _ => None,
                    }
                },
                _ => None,
            }
        },
        _ => None,
    }
}

pub fn compare(a: &LuaValue, b: &LuaValue, op: u8) -> Option<bool> {
    match op {
        LUA_OPEQ => Some(eq(a, b)),
        LUA_OPLT => lt(a, b),
        LUA_OPLE => le(a, b),
        _ => None,
    }
}

macro_rules! cmp {
    ($a:ident $op:tt $b:ident) => {
        match $a {
            LuaValue::Str(x) => match $b {
                LuaValue::Str(y) => Some(x $op y),
                _ => None,
            },
            LuaValue::Integer(x) => match $b {
                LuaValue::Integer(y) => Some(x $op y),
                LuaValue::Number(y) => Some((*x as f64) $op *y),
                _ => None,
            },
            LuaValue::Number(x) => match $b {
                LuaValue::Number(y) => Some(x $op y),
                LuaValue::Integer(y) => Some(*x $op (*y as f64)),
                _ => None,
            },
            _ => None,
        }
    }
}

pub fn eq(a: &LuaValue, b: &LuaValue) -> bool {
    if let Some(x) = cmp!(a == b) {
        x
    } else {
        match a {
            LuaValue::Nil => match b {
                LuaValue::Nil => true,
                _ => false,
            },
            LuaValue::Bool(x) => match b {
                LuaValue::Bool(y) => x == y,
                _ => false,
            },
            _ => false,
        }
    }
}

fn lt(a: &LuaValue, b: &LuaValue) -> Option<bool> {
    cmp!(a < b)
}

fn le(a: &LuaValue, b: &LuaValue) -> Option<bool> {
    cmp!(a <= b)
}