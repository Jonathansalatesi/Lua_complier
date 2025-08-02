use crate::api::{lua_vm::LuaVM, consts::*};
use super::instruction::*;

/* arith */
pub fn add(i: &Instruction, vm: &mut dyn LuaVM) {
    binary_arith(i, vm, LUA_OPADD)
} // +
pub fn sub(i: &Instruction, vm: &mut dyn LuaVM) {
    binary_arith(i, vm, LUA_OPSUB)
} // -
pub fn mul(i: &Instruction, vm: &mut dyn LuaVM) {
    binary_arith(i, vm, LUA_OPMUL)
} // *
pub fn _mod(i: &Instruction, vm: &mut dyn LuaVM) {
    binary_arith(i, vm, LUA_OPMOD)
} // %
pub fn pow(i: &Instruction, vm: &mut dyn LuaVM) {
    binary_arith(i, vm, LUA_OPPOW)
} // ^
pub fn div(i: &Instruction, vm: &mut dyn LuaVM) {
    binary_arith(i, vm, LUA_OPDIV)
} // /
pub fn idiv(i: &Instruction, vm: &mut dyn LuaVM) {
    binary_arith(i, vm, LUA_OPIDIV)
} // //
pub fn band(i: &Instruction, vm: &mut dyn LuaVM) {
    binary_arith(i, vm, LUA_OPBAND)
} // &
pub fn bor(i: &Instruction, vm: &mut dyn LuaVM) {
    binary_arith(i, vm, LUA_OPBOR)
} // |
pub fn bxor(i: &Instruction, vm: &mut dyn LuaVM) {
    binary_arith(i, vm, LUA_OPBXOR)
} // ~
pub fn shl(i: &Instruction, vm: &mut dyn LuaVM) {
    binary_arith(i, vm, LUA_OPSHL)
} // <<
pub fn shr(i: &Instruction, vm: &mut dyn LuaVM) {
    binary_arith(i, vm, LUA_OPSHR)
} // >>
pub fn unm(i: &Instruction, vm: &mut dyn LuaVM) {
    unary_arith(i, vm, LUA_OPUNM)
} // -
pub fn bnot(i: &Instruction, vm: &mut dyn LuaVM) {
    unary_arith(i, vm, LUA_OPBNOT)
} // ~

// R(A) := RK(B) op RK(C)
fn binary_arith(i: &Instruction, vm: &mut dyn LuaVM, op: u8) {
    let (mut a, b, c) = i.ABC();
    a += 1;

    vm.GetRK(b);
    vm.GetRK(c);
    vm.ArithOp(op);
    vm.Replace(a);
}

// R(A) := op R(B)
fn unary_arith(i: &Instruction, vm: &mut dyn LuaVM, op: u8) {
    let (mut a, mut b, _) = i.ABC();
    a += 1;
    b += 1;

    vm.PushValue(b);
    vm.ArithOp(op);
    vm.Replace(a);
}

/* compare */
pub fn eq(i: &Instruction, vm: &mut dyn LuaVM) {
    compare(i, vm, LUA_OPEQ)
} // ==
pub fn lt(i: &Instruction, vm: &mut dyn LuaVM) {
    compare(i, vm, LUA_OPLT)
} // <
pub fn le(i: &Instruction, vm: &mut dyn LuaVM) {
    compare(i, vm, LUA_OPLE)
} // <=

// if ((RK(B) op RK(C)) ~= A) then pc++
fn compare(i: &Instruction, vm: &mut dyn LuaVM, op: u8) {
    let (a, b, c) = i.ABC();

    vm.GetRK(b);
    vm.GetRK(c);
    if vm.Compare(-2, -1, op) != (a != 0) {
        vm.AddPC(1);
    }
    vm.pop(2);
}

/* logical */

// R(A) := not R(B)
pub fn not(i: &Instruction, vm: &mut dyn LuaVM) {
    let (mut a, mut b, _) = i.ABC();
    a += 1;
    b += 1;

    vm.PushBoolean(!vm.ToBoolean(b));
    vm.Replace(a);
}

// if not (R(A) <=> C) then pc++
pub fn test(i: &Instruction, vm: &mut dyn LuaVM) {
    let (mut a, _, c) = i.ABC();
    a += 1;

    if vm.ToBoolean(a) != (c != 0) {
        vm.AddPC(1);
    }
}

// if (R(B) <=> C) then R(A) := R(B) else pc++
pub fn testSet(i: &Instruction, vm: &mut dyn LuaVM) {
    let (mut a, mut b, c) = i.ABC();
    a += 1;
    b += 1;

    if vm.ToBoolean(b) == (c != 0) {
        vm.copy(b, a);
    } else {
        vm.AddPC(1);
    }
}

/* len & concat */

// R(A) := length of R(B)
pub fn _len(i: &Instruction, vm: &mut dyn LuaVM) {
    let (mut a, mut b, _) = i.ABC();
    a += 1;
    b += 1;

    vm.Len(b);
    vm.Replace(a);
}

// R(A) := R(B).. ... ..R(C)
pub fn concat(i: &Instruction, vm: &mut dyn LuaVM) {
    let (mut a, mut b, mut c) = i.ABC();
    a += 1;
    b += 1;
    c += 1;

    let n = c - b + 1;
    vm.CheckStack(n);
    for i in b..(c + 1) {
        vm.PushValue(i);
    }
    vm.Concat(n);
    vm.Replace(a);
}