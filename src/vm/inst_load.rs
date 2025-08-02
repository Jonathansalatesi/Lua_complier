use crate::api::lua_vm::LuaVM;
use super::instruction::Instruction;

pub fn loadNil(i: &Instruction, vm: &mut dyn LuaVM) {
    let (mut a, b, _) = i.ABC();
    a += 1;
    vm.PushNil();
    for i in a..=(a + b) {
        vm.copy(-1, i);
    }
    vm.pop(1);
}

pub fn loadBool(i: &Instruction, vm: &mut dyn LuaVM) {
    let (mut a, b, c) = i.ABC();
    a += 1;
    vm.PushBoolean(b != 0);
    vm.Replace(a);
    if c != 0 {
        vm.AddPC(1);
    }
}

pub fn loadK(i: &Instruction, vm: &mut dyn LuaVM) {
    let (mut a, bx) = i.ABx();
    a += 1;
    vm.GetConst(bx);
    vm.Replace(a);
}

pub fn loadKx(i: &Instruction, vm: &mut dyn LuaVM) {
    let (mut a, _) = i.ABx();
    a += 1;
    let ax = Instruction::new(vm.Fetch()).Ax();
    vm.GetConst(ax);
    vm.Replace(a);
}