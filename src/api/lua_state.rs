use crate::{state::lua_state::LuaState, api::consts::*};

pub type RustFn = fn(&mut LuaState) -> i32;

pub fn LuaUpValueIndex(i: i32) -> i32 {
    LUA_REGISTRYINDEX as i32 - i
}

pub trait LuaAPI {
    /* basic stack manipulation */
    fn GetTop(&self) -> i32;
    fn AbsIndex(&self, idx: i32) -> i32;
    fn CheckStack(&mut self, n: i32) -> bool;
    fn pop(&mut self, n: i32);
    fn copy(&mut self, from_idx: i32, to_idx: i32);
    fn PushValue(&mut self, idx: i32);
    fn Replace(&mut self, idx: i32);
    fn Insert(&mut self, idx: i32);
    fn Remove(&mut self, idx: i32);
    fn Rotate(&mut self, idx: i32, n: i32);
    fn SetTop(&mut self, idx: i32);
    /* access functions (stack -> rust) */
    fn TypeName(&self, tp: i8) -> &'static str; // TODO
    fn Type(&self, idx: i32) -> i8; // `type` is a keyword
    fn IsNone(&self, idx: i32) -> bool;
    fn IsNil(&self, idx: i32) -> bool;
    fn IsNoneOrNil(&self, idx: i32) -> bool;
    fn IsBoolean(&self, idx: i32) -> bool;
    fn IsInteger(&self, idx: i32) -> bool;
    fn IsNumber(&self, idx: i32) -> bool;
    fn IsString(&self, idx: i32) -> bool;
    fn IsTable(&self, idx: i32) -> bool;
    fn IsThread(&self, idx: i32) -> bool;
    fn IsFunction(&self, idx: i32) -> bool;
    fn ToBoolean(&self, idx: i32) -> bool;
    fn ToInteger(&self, idx: i32) -> i64;
    fn ToIntegerX(&self, idx: i32) -> Option<i64>;
    fn ToNumber(&self, idx: i32) -> f64;
    fn ToNumberX(&self, idx: i32) -> Option<f64>;
    fn ToString(&self, idx: i32) -> String;
    fn ToStringX(&self, idx: i32) -> Option<String>;
    /* push functions (rust -> stack) */
    fn PushNil(&mut self);
    fn PushBoolean(&mut self, b: bool);
    fn PushInteger(&mut self, n: i64);
    fn PushNumber(&mut self, n: f64);
    fn PushString(&mut self, s: String);

    fn ArithOp(&mut self, op: u8);
    fn Compare(&mut self, idx1: i32, idx2: i32, op: u8) -> bool;
    fn Len(&mut self, idx: i32);
    fn Concat(&mut self, n: i32);

    // get function
    fn NewTable(&mut self);
    fn CreateTable(&mut self, nArr: i32, nRec: i32);
    fn GetTable(&mut self, idx: i32) -> i8;
    fn GetField(&mut self, idx: i32, k: &'static str) -> i8;
    fn GetI(&mut self, idx: i32, i: i64) -> i8;

    // set function
    fn SetTable(&mut self, idx: i32);
    fn SetField(&mut self, idx: i32, k: &'static str);
    fn SetI(&mut self, idx: i32, n: i64);

    // closure
    fn Load(&mut self, chunk: Vec<u8>, chunkName: &str, mode: &str) -> i32;
    fn Call(&mut self, nArgs: i32, nResults: i32);

    // rust function
    fn PushRustFunction(&mut self, f: RustFn);
    fn IsRustFunction(&self, idx: i32) -> bool;
    fn ToGoFunction(&self, idx: i32) -> Option<RustFn>;

    // global environment
    fn PushGlobalTable(&mut self);
    fn GetGlobal(&mut self, name: &'static str) -> i8;
    fn SetGlobal(&mut self, name: &'static str);
    fn Register(&mut self, name: &'static str, f: RustFn);

    // ch10 added
    fn PushGoClosure(&mut self, f: RustFn, n: i32);

    // ch11 added
    fn GetMetatable(&mut self, idx: i32) -> bool;
    fn SetMetatable(&mut self, idx: i32);
    fn RawLen(&self, idx: i32) -> u32;
    fn RawEqual(&self, idx1: i32, idx2: i32) -> bool;
    fn RawGet(&mut self, idx: i32) -> i8;
    fn RawSet(&mut self, idx: i32);
    fn RawGetI(&mut self, idx: i32, i: i64) -> i8;
    fn RawSetI(&mut self, idx: i32, i: i64);

    // ch12 added
    fn Next(&mut self, idx: i32) -> bool;
}