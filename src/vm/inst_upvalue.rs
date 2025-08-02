use crate::api::{lua_state::LuaUpValueIndex, lua_vm::LuaVM};
use super::instruction::*;

pub fn getTabUp(i: &Instruction, vm: &mut dyn LuaVM) {
    let (mut a, mut b, c) = i.ABC();
    a += 1;
    b += 1;

    vm.GetRK(c);
    vm.GetTable(LuaUpValueIndex(b));
    vm.Replace(a);
}

pub fn setTabUp(i: &Instruction, vm: &mut dyn LuaVM) {
    let (mut a, b, c) = i.ABC();
    a += 1;

    vm.GetRK(b);
    vm.GetRK(c);
    vm.SetTable(LuaUpValueIndex(a));
}

pub fn getUpVal(i: &Instruction, vm: &mut dyn LuaVM) {
    let (mut a, mut b, _) = i.ABC();
    a += 1;
    b += 1;
    vm.copy(LuaUpValueIndex(b), a);
}

pub fn setUpVal(i: &Instruction, vm: &mut dyn LuaVM) {
    let (mut a, mut b, _) = i.ABC();
    a += 1;
    b += 1;
    vm.copy(a, LuaUpValueIndex(b));
}