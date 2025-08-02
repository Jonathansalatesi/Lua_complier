use crate::api::lua_vm::LuaVM;
use super::instruction::*;

pub fn move_(i: &Instruction, vm: &mut dyn LuaVM) {
    let (a, b, _) = i.ABC();
    vm.copy(b + 1, a + 1);
}

pub fn jmp(i: &Instruction, vm: &mut dyn LuaVM) {
    let (a, sBx) = i.AsBx();
    vm.AddPC(sBx);
    if a != 0 {
        vm.CloseUpvalues(a);
    }
}