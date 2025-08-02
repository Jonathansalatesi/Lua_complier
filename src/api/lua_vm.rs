use super::lua_state::LuaAPI;

pub trait LuaVM: LuaAPI {
    fn PC(&self) -> i32;
    fn AddPC(&mut self, n: i32);
    fn Fetch(&mut self) -> u32;
    fn GetConst(&mut self, idx: i32);
    fn GetRK(&mut self, rk: i32);
    fn RegisterCount(&self) -> i32;
    fn LoadVararg(&mut self, n: i32);
    fn LoadProto(&mut self, idx: i32);
    fn CloseUpvalues(&mut self, a: i32);
}