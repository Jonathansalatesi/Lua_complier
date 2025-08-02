use crate::api::lua_vm::LuaVM;
use super::instruction::Instruction;

pub fn closure(i: &Instruction, vm: &mut dyn LuaVM) {
    let (mut a, bx) = i.ABx();
    a += 1;
    vm.LoadProto(bx);
    vm.Replace(a);
}

pub fn call(i: &Instruction, vm: &mut dyn LuaVM) {
    let (mut a, b, c) = i.ABC();
    a += 1;

    let nArgs = _pushFuncAndArgs(a, b, vm);
    vm.Call(nArgs, c - 1);
    _popResults(a, c, vm);
}

fn _popResults(a: i32, c: i32, vm: &mut dyn LuaVM) {
    if c == 1 {     // no result
    } else if c > 1 {       // c - 1 results
        for i in (a..=(a + c - 2)).rev() {
            vm.Replace(i);
        }
    } else {
        vm.CheckStack(1);
        vm.PushInteger(a as i64);
    }
}

fn _pushFuncAndArgs(a: i32, b: i32, vm: &mut dyn LuaVM) -> i32 {
    if b >= 1 {         // b - 1 args
        vm.CheckStack(b);
        for i in a..(a + b) {
            vm.PushValue(i);
        }
        return b - 1;
    } else {
        _fixStack(a, vm);
        return vm.GetTop() - vm.RegisterCount() - 1;
    }
}

fn _fixStack(a: i32, vm: &mut dyn LuaVM) {
    let x = vm.ToInteger(-1) as i32;
    vm.pop(1);
    vm.CheckStack(x - a);
    for i in a..x {
        vm.PushValue(i);
    }
    vm.Rotate(vm.RegisterCount() + 1, x - a);
}

pub fn _return(i: &Instruction, vm: &mut dyn LuaVM) {
    let (mut a, b, _) = i.ABC();
    a += 1;
    if b == 1 {     // no return value
    } else if b > 1 {       // b - 1 return values
        vm.CheckStack(b - 1);
        for i in a..=(a + b - 2) {
            vm.PushValue(i);
        }
    } else {
        _fixStack(a, vm);
    }
}

pub fn vararg(i: &Instruction, vm: &mut dyn LuaVM) {
    let (mut a, b, _) = i.ABC();
    a += 1;
    if b != 1 {
        vm.LoadVararg(b - 1);
        _popResults(a, b, vm);
    }
}

pub fn tailcall(i: &Instruction, vm: &mut dyn LuaVM) {
    let (mut a, b, _) = i.ABC();
    a += 1;
    let c = 0;
    let nArgs = _pushFuncAndArgs(a, b, vm);
    vm.Call(nArgs, c - 1);
    _popResults(a, c, vm);
}

pub fn _self(i: &Instruction, vm: &mut dyn LuaVM) {
    let (mut a, mut b, c) = i.ABC();
    a += 1;
    b += 1;
    vm.copy(b, a + 1);
    vm.GetRK(c);
    vm.GetTable(b);
    vm.Replace(a);
}

pub fn tForCall(i: &Instruction, vm: &mut dyn LuaVM) {
    let (mut a, _, c) = i.ABC();
    a += 1;
    _pushFuncAndArgs(a, 3, vm);
    vm.Call(2, c);
    _popResults(a + 3, c + 1, vm);
}

pub fn tForLoop(i: &Instruction, vm: &mut dyn LuaVM) {
    let (mut a, sBx) = i.AsBx();
    a += 1;

    if !vm.IsNil(a + 1) {
        vm.copy(a + 1, a);
        vm.AddPC(sBx);
    }
}