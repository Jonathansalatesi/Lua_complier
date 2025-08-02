use std::{fmt, cell::RefCell, rc::Rc, hash::{Hash, Hasher}};

use crate::{api::{consts, lua_state::{LuaAPI, RustFn}}, number::parser::ParseInteger};
use super::{closure::Closure, lua_state::LuaState, lua_table::LuaTable};

#[derive(Clone)]
pub enum LuaValue {
    Nil,
    Bool(bool),
    Integer(i64),
    Number(f64),
    Str(String),
    Table(Rc<RefCell<LuaTable>>),
    Function(Rc<Closure>),
}

impl fmt::Debug for LuaValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LuaValue::Nil => write!(f, "(nil)"),
            LuaValue::Bool(b) => write!(f, "({})", b),
            LuaValue::Integer(i) => write!(f, "({})", i),
            LuaValue::Number(n) => write!(f, "({})", n),
            LuaValue::Str(s) => write!(f, "({})", s),
            LuaValue::Table(tbl) => write!(f, "({:?})", tbl),
            LuaValue::Function(_) => write!(f, "(closure)"),
        }
    }
}

impl PartialEq for LuaValue {
    fn eq(&self, other: &LuaValue) -> bool {
        if let (LuaValue::Nil, LuaValue::Nil) = (self, other) {
            true
        } else if let (LuaValue::Bool(x), LuaValue::Bool(y)) = (self, other) {
            x == y
        } else if let (LuaValue::Integer(x), LuaValue::Integer(y)) = (self, other) {
            x == y
        } else if let (LuaValue::Number(x), LuaValue::Number(y)) = (self, other) {
            x == y // TODO
        } else if let (LuaValue::Str(x), LuaValue::Str(y)) = (self, other) {
            x == y
        } else if let (LuaValue::Table(x), LuaValue::Table(y)) = (self, other) {
            Rc::ptr_eq(x, y)
        } else if let (LuaValue::Function(x), LuaValue::Function(y)) = (self, other) {
            Rc::ptr_eq(x, y)
        }else {
            false
        }
    }
}

// the trait `std::cmp::Eq` is not implemented for `f64`
impl Eq for LuaValue {} // TODO

// the trait `std::hash::Hash` is not implemented for `f64`
impl Hash for LuaValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            LuaValue::Nil => 0.hash(state),
            LuaValue::Bool(b) => b.hash(state),
            LuaValue::Integer(i) => i.hash(state),
            LuaValue::Number(n) => n.to_bits().hash(state),
            LuaValue::Str(s) => s.hash(state),
            LuaValue::Table(t) => t.borrow().hash(state),
            LuaValue::Function(f) => f.hash(state),
        }
    }
}

impl LuaValue {
    pub fn typeOf(&self) -> i8 {
        match self {
            Self::Nil => consts::LUA_TNIL,
            Self::Bool(_) => consts::LUA_TBOOLEAN,
            Self::Integer(_) => consts::LUA_TNUMBER,
            Self::Number(_) => consts::LUA_TNUMBER,
            Self::Str(_) => consts::LUA_TSTRING,
            Self::Table(_) => consts::LUA_TTABLE,
            Self::Function(_) => consts::LUA_TFUNCTION,
            _ => {
                panic!("todo!");
            }
        }
    }

    pub fn ToBoolean(&self) -> bool {
        match self {
            Self::Nil => false,
            Self::Bool(x) => *x,
            _ => true,
        }
    }

    pub fn ToFloat(&self) -> Option<f64> {
        match self {
            LuaValue::Number(n) => Some(*n),
            LuaValue::Integer(i) => Some(*i as f64),
            LuaValue::Str(s) => {
                match s.parse::<f64>() {
                    Ok(val) => Some(val),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn ToInteger(&self) -> Option<i64> {
        match self {
            LuaValue::Integer(i) => Some(*i),
            LuaValue::Number(n) => Some(*n as i64),
            LuaValue::Str(s) => {
                let (val, b) = ParseInteger(s);
                if b {
                    Some(val)
                } else {
                    None
                }
            },
            _ => None,
        }
    }

    pub fn IsNil(&self) -> bool {
        match self {
            LuaValue::Nil => true,
            _ => false,
        }
    }

    pub fn newRustClosure(f: RustFn, n_upvals: i32) -> Self {
        Self::Function(Rc::new(Closure::newRustClosure(f, n_upvals)))
    }
}

pub fn setMetatable(val: LuaValue, mt: Option<Rc<RefCell<LuaTable>>>, ls: &mut LuaState) {
    if let LuaValue::Table(tbl) = &val {
        tbl.borrow_mut().metatable = mt;
        return;
    }
    let _key_ = format!("_MT{}", val.typeOf());
    if let LuaValue::Table(tbl) = &ls.registry {
        tbl.borrow_mut().Put(LuaValue::Str(_key_), val);
    }
}

pub fn getMetatable(val: LuaValue, ls: &mut LuaState) -> LuaValue {
    if let LuaValue::Table(tbl) = &val {
        if let Some(r_meta) = &tbl.borrow().metatable {
            return LuaValue::Table(Rc::clone(r_meta));
        }
    }
    let _key_ = LuaValue::Str(format!("_MT{}", val.typeOf()));
    if let LuaValue::Table(tbl) = &ls.registry {
        return tbl.borrow().Get(&_key_);
    }
    LuaValue::Nil
}

pub fn getMetafield(val: LuaValue, fieldName: &str, ls: &mut LuaState) -> LuaValue {
    if let LuaValue::Table(tbl) = getMetatable(val, ls) {
        return tbl.borrow().Get(&LuaValue::Str(String::from(fieldName)));
    }
    LuaValue::Nil
}

pub fn callMetamethod(a: LuaValue, b: LuaValue, mmName: &str, ls: &mut LuaState) -> Option<LuaValue> {
    let _mm1_ = getMetafield(a.clone(), mmName, ls);
    let _mm2_ = getMetafield(b.clone(), mmName, ls);
    let mut _mm_ = _mm1_.clone();
    if let LuaValue::Nil = _mm1_ {
        _mm_ = _mm2_.clone();
        if let LuaValue::Nil = _mm2_ {
            return None;
        }
    }
    ls.stack_mut().check(4);
    ls.stack_mut().push(_mm_);
    ls.stack_mut().push(a);
    ls.stack_mut().push(b);
    ls.Call(2, 1);
    Some(ls.stack_mut().pop())
}

#[cfg(test)]
mod tests {
    use crate::state::lua_value::LuaValue;

    #[test]
    fn test_partial_eq() {
        let a = LuaValue::Str("10".to_string());
        let b = LuaValue::Str("10".to_string());
        assert_eq!(&a, &b);
    }
}