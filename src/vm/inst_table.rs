use crate::api::lua_vm::LuaVM;
use super::{instruction::Instruction, fpb::*};

const LFIELDS_PER_FLUSH: i64 = 50;

pub fn newTable(i: &Instruction, vm: &mut dyn LuaVM) {
    let (mut a, b, c) = i.ABC();
    a += 1;
    vm.CreateTable(fb2int(b as usize) as i32, fb2int(c as usize) as i32);
    vm.Replace(a);
}

pub fn getTable(i: &Instruction, vm: &mut dyn LuaVM) {
    let (mut a, mut b, c) = i.ABC();
    a += 1;
    b += 1;
    vm.GetRK(c);
    vm.GetTable(b);
    vm.Replace(a);
}

pub fn setTable(i: &Instruction, vm: &mut dyn LuaVM) {
    let (mut a, b, c) = i.ABC();
    a += 1;
    vm.GetRK(b);
    vm.GetRK(c);
    vm.SetTable(a);
}

pub fn setList(i: &Instruction, vm: &mut dyn LuaVM) {
    let (mut a, mut b, mut c) = i.ABC();
    a += 1;
    if c > 0 {
        c = c - 1;
    } else {
        c = Instruction::new(vm.Fetch()).Ax();
    }

    vm.CheckStack(1);

    let b_is_zero = b == 0;
    if b_is_zero {
        b = vm.ToInteger(-1) as i32 - a - 1;
        vm.pop(1);
    }

    let mut idx = c as i64 * LFIELDS_PER_FLUSH;
    for j in 1..=b {
        idx += 1;
        vm.PushValue(a + j);
        vm.SetI(a, idx);            // TODO!
    }

    if b_is_zero {
        let nreg = vm.RegisterCount();
        for j in (nreg + 1)..(vm.GetTop() + 1) {
            idx += 1;
            vm.PushValue(j);
            vm.SetI(a, idx);
        }

        // clear stack
        vm.SetTop(nreg);
    }
}