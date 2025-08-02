use std::{cell::RefCell, collections::HashMap, hash::Hash, rc::Rc};
use crate::number::math::{FloatToInteger, random};
use super::lua_value::LuaValue;

#[derive(Debug)]
pub struct LuaTable {
    arr: Vec<LuaValue>,
    _map: HashMap<LuaValue, LuaValue>,
    pub metatable: Option<Rc<RefCell<LuaTable>>>,
    rdm: usize,                         // for Hash
    keys: Option<HashMap<LuaValue, LuaValue>>,
    changed: bool,
    lastKey: Option<LuaValue>,
}

impl Hash for LuaTable {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.rdm.hash(state);
    }
}

fn to_index(key: &LuaValue) -> Option<usize> {
    if let LuaValue::Integer(i) = key {
        if *i >= 1 {
            return Some(*i as usize);
        }
    } else if let LuaValue::Number(n) = key {
        let (i, is_ok) = FloatToInteger(*n);
        if is_ok {
            if i >= 1 {
                return Some(i as usize);
            }
        }
    }
    None
}

impl LuaTable {
    pub fn new(nArr: i32, nRec: i32) -> Self {
        if nArr > 0 {
            LuaTable {
                arr: Vec::with_capacity(nArr as usize),
                _map: HashMap::new(),
                metatable: None,
                rdm: random(),
                keys: None,
                changed: false,
                lastKey: None,
            }
        } else if nRec > 0 {
            LuaTable {
                arr: Vec::new(),
                _map: HashMap::with_capacity(nRec as usize),
                metatable: None,
                rdm: random(),
                keys: None,
                changed: false,
                lastKey: None,
            }
        } else {
            LuaTable {
                arr: Vec::new(),
                _map: HashMap::new(),
                metatable: None,
                rdm: random(),
                keys: None,
                changed: false,
                lastKey: None,
            }
        }
    }

    pub fn Len(&self) -> usize {
        if self.arr.len() > self._map.len() {
            self.arr.len()
        } else {
            self._map.len()
        }
    }

    pub fn Get(&self, key: &LuaValue) -> LuaValue {
        if let Some(idx) = to_index(key) {
            if idx <= self.arr.len() {
                return self.arr[idx - 1].clone(); // TODO
            }
        }
        if let Some(val) = self._map.get(key) {
            val.clone() // TODO
        } else {
            LuaValue::Nil
        }
    }

    pub fn Put(&mut self, key: LuaValue, val: LuaValue) {
        if key.IsNil() {
            panic!("table index is nil!");
        }
        if let LuaValue::Number(n) = key {
            if n.is_nan() {
                panic!("table index is NaN!");
            }
        }

        if let Some(idx) = to_index(&key) {
            let arr_len = self.arr.len();
            if idx <= arr_len {
                let val_is_nil = val.IsNil();
                self.arr[idx - 1] = val;
                if idx == arr_len && val_is_nil {
                    self.ShrinkArray();
                }
                return;
            }
            if idx == arr_len + 1 {
                self._map.remove(&key);
                if !val.IsNil() {
                    self.arr.push(val);
                    self.ExpandArray();
                }
                return;
            }
        }

        if !val.IsNil() {
            self._map.insert(key, val);
        } else {
            self._map.remove(&key);
        }
    }

    fn ShrinkArray(&mut self) {
        while !self.arr.is_empty() {
            if self.arr.last().unwrap().IsNil() {
                self.arr.pop();
            } else {
                break;
            }
        }
    }

    fn ExpandArray(&mut self) {
        let mut idx = self.arr.len() + 1;
        loop {
            let key = LuaValue::Integer(idx as i64);
            if self._map.contains_key(&key) {
                let val = self._map.remove(&key).unwrap();
                self.arr.push(val);
                idx += 1;
            } else {
                break;
            }
        }
    }

    pub fn hasMetafield(&self, fieldName: &str) -> bool {
        match &self.metatable {
            None => false,
            Some(_tbl_) => {
                !_tbl_.borrow().Get(&LuaValue::Str(String::from(fieldName))).IsNil()
            },
        }
    }

    pub fn nextKey(&mut self, key: &LuaValue) -> LuaValue {
        if self.keys.is_none() || (key.IsNil() && self.changed) {
            self.initKeys();
            self.changed = false;
        }

        let next_key = self.keys.as_ref().unwrap().get(key);
        if next_key.is_none() && !key.IsNil() && *key != *self.lastKey.as_ref().unwrap() {
            panic!("invalid key to 'next'");
        }

        if let Some(val) = next_key {
            return val.clone();
        } else {
            return LuaValue::Nil;
        }
    }

    pub fn initKeys(&mut self) {
        self.keys = Some(HashMap::new());
        let mut key = LuaValue::Nil;
        for (i, v) in self.arr.iter().enumerate() {
            if !v.IsNil() {
                self.keys.as_mut().unwrap().insert(key.clone(), LuaValue::Integer(i as i64 + 1));
                key = LuaValue::Integer(i as i64 + 1);
            }
        }

        for (k, v) in self._map.iter() {
            if !v.IsNil() {
                self.keys.as_mut().unwrap().insert(key.clone(), k.clone());
                key = k.clone();
            }
        }
        self.lastKey = Some(key);
    }
}

pub fn newLuaTable(nArr: i32, nRec: i32) -> LuaValue {
    LuaValue::Table(newTable(nArr, nRec))
}

pub fn newTable(nArr: i32, nRec: i32) -> Rc<RefCell<LuaTable>> {
    Rc::new(RefCell::new(LuaTable::new(nArr, nRec)))
}