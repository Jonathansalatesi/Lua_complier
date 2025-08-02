use crate::api::{lua_vm::LuaVM, consts::*};
use super::instruction::*;

// R(A)-=R(A+2); pc+=sBx
pub fn for_prep(i: &Instruction, vm: &mut dyn LuaVM) {
    let (mut a, sbx) = i.AsBx();
    a += 1;

    if vm.Type(a) == LUA_TSTRING {
        vm.PushNumber(vm.ToNumber(a));
        vm.Replace(a);
    }
    if vm.Type(a + 1) == LUA_TSTRING {
        vm.PushNumber(vm.ToNumber(a + 1));
        vm.Replace(a + 1);
    }
    if vm.Type(a + 2) == LUA_TSTRING {
        vm.PushNumber(vm.ToNumber(a + 2));
        vm.Replace(a + 2);
    }

    vm.PushValue(a);
    vm.PushValue(a + 2);
    vm.ArithOp(LUA_OPSUB);
    vm.Replace(a);
    vm.AddPC(sbx);
}

// R(A)+=R(A+2);
// if R(A) <?= R(A+1) then {
//   pc+=sBx; R(A+3)=R(A)
// }
pub fn for_loop(i: &Instruction, vm: &mut dyn LuaVM) {
    let (mut a, sbx) = i.AsBx();
    a += 1;

    // R(A)+=R(A+2);
    vm.PushValue(a + 2);
    vm.PushValue(a);
    vm.ArithOp(LUA_OPADD);
    vm.Replace(a);

    let positive_step = vm.ToNumber(a + 2) >= 0.0;
    if positive_step && vm.Compare(a, a + 1, LUA_OPLE) || !positive_step && vm.Compare(a + 1, a, LUA_OPLE) {
        // pc+=sBx; R(A+3)=R(A)
        vm.AddPC(sbx);
        vm.copy(a, a + 3);
    }
}