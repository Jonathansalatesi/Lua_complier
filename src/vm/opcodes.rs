#[derive(Clone)]
pub enum Mode {
    IABC,
    IABx,
    IAsBx,
    IAx,
}

// op code
pub const OP_MOVE: u8 = 0x00;
pub const OP_LOADK: u8 = 0x01;
pub const OP_LOADKX: u8 = 0x02;
pub const OP_LOADBOOL: u8 = 0x03;
pub const OP_LOADNIL: u8 = 0x04;
pub const OP_GETUPVAL: u8 = 0x05;
pub const OP_GETTABUP: u8 = 0x06;
pub const OP_GETTABLE: u8 = 0x07;
pub const OP_SETTABUP: u8 = 0x08;
pub const OP_SETUPVAL: u8 = 0x09;
pub const OP_SETTABLE: u8 = 0x0a;
pub const OP_NEWTABLE: u8 = 0x0b;
pub const OP_SELF: u8 = 0x0c;
pub const OP_ADD: u8 = 0x0d;
pub const OP_SUB: u8 = 0x0e;
pub const OP_MUL: u8 = 0x0f;
pub const OP_MOD: u8 = 0x10;
pub const OP_POW: u8 = 0x11;
pub const OP_DIV: u8 = 0x12;
pub const OP_IDIV: u8 = 0x13;
pub const OP_BAND: u8 = 0x14;
pub const OP_BOR: u8 = 0x15;
pub const OP_BXOR: u8 = 0x16;
pub const OP_SHL: u8 = 0x17;
pub const OP_SHR: u8 = 0x18;
pub const OP_UNM: u8 = 0x19;
pub const OP_BNOT: u8 = 0x1a;
pub const OP_NOT: u8 = 0x1b;
pub const OP_LEN: u8 = 0x1c;
pub const OP_CONCAT: u8 = 0x1d;
pub const OP_JMP: u8 = 0x1e;
pub const OP_EQ: u8 = 0x1f;
pub const OP_LT: u8 = 0x20;
pub const OP_LE: u8 = 0x21;
pub const OP_TEST: u8 = 0x22;
pub const OP_TESTSET: u8 = 0x23;
pub const OP_CALL: u8 = 0x24;
pub const OP_TAILCALL: u8 = 0x25;
pub const OP_RETURN: u8 = 0x26;
pub const OP_FORLOOP: u8 = 0x27;
pub const OP_FORPREP: u8 = 0x28;
pub const OP_TFORCALL: u8 = 0x29;
pub const OP_TFORLOOP: u8 = 0x2a;
pub const OP_SETLIST: u8 = 0x2b;
pub const OP_CLOSURE: u8 = 0x2c;
pub const OP_VARARG: u8 = 0x2d;
pub const OP_EXTRAARG: u8 = 0x2e;

#[derive(Clone)]
pub enum OpArg {
    OpArgN,     // argument is not used
    OpArgU,     // argument is used
    OpArgR,     // argument is a register or a jump offset
    OpArgK,     // argument is a constant or register/constant
}

pub struct Opcode {
    pub testFlag: u8,       // operator is a test (next instruction must be a jump)
    pub setAFlag: u8,       // instruction set register A
    pub argBMode: OpArg,
    pub argCMode: OpArg,
    pub opMode: Mode,
    pub name: &'static str,
}

const fn opcode(T: u8, A: u8, B: OpArg, C: OpArg, mode: Mode, name: &'static str) -> Opcode {
    Opcode {
        testFlag: T,
        setAFlag: A,
        argBMode: B,
        argCMode: C,
        opMode: mode,
        name: name,
    }
}

use OpArg::*;
use Mode::*;

pub const OPCODES: &'static [Opcode] = &[
    //        T    A      B         C            mode        name
    opcode(0, 1, OpArgR, OpArgN, IABC, "MOVE    "), // R(A) := R(B)
    opcode(0, 1, OpArgK, OpArgN, IABx, "LOADK   "), // R(A) := Kst(Bx)
    opcode(0, 1, OpArgN, OpArgN, IABx, "LOADKX  "), // R(A) := Kst(extra arg)
    opcode(0, 1, OpArgU, OpArgU, IABC, "LOADBOOL"), // R(A) := (bool)B; if (C) pc++
    opcode(0, 1, OpArgU, OpArgN, IABC, "LOADNIL "), // R(A), R(A+1), ..., R(A+B) := nil
    opcode(0, 1, OpArgU, OpArgN, IABC, "GETUPVAL"), // R(A) := UpValue[B]
    opcode(0, 1, OpArgU, OpArgK, IABC, "GETTABUP"), // R(A) := UpValue[B][RK(C)]
    opcode(0, 1, OpArgR, OpArgK, IABC, "GETTABLE"), // R(A) := R(B)[RK(C)]
    opcode(0, 0, OpArgK, OpArgK, IABC, "SETTABUP"), // UpValue[A][RK(B)] := RK(C)
    opcode(0, 0, OpArgU, OpArgN, IABC, "SETUPVAL"), // UpValue[B] := R(A)
    opcode(0, 0, OpArgK, OpArgK, IABC, "SETTABLE"), // R(A)[RK(B)] := RK(C)
    opcode(0, 1, OpArgU, OpArgU, IABC, "NEWTABLE"), // R(A) := {} (size = B,C)
    opcode(0, 1, OpArgR, OpArgK, IABC, "SELF    "), // R(A+1) := R(B); R(A) := R(B)[RK(C)]
    opcode(0, 1, OpArgK, OpArgK, IABC, "ADD     "), // R(A) := RK(B) + RK(C)
    opcode(0, 1, OpArgK, OpArgK, IABC, "SUB     "), // R(A) := RK(B) - RK(C)
    opcode(0, 1, OpArgK, OpArgK, IABC, "MUL     "), // R(A) := RK(B) * RK(C)
    opcode(0, 1, OpArgK, OpArgK, IABC, "MOD     "), // R(A) := RK(B) % RK(C)
    opcode(0, 1, OpArgK, OpArgK, IABC, "POW     "), // R(A) := RK(B) ^ RK(C)
    opcode(0, 1, OpArgK, OpArgK, IABC, "DIV     "), // R(A) := RK(B) / RK(C)
    opcode(0, 1, OpArgK, OpArgK, IABC, "IDIV    "), // R(A) := RK(B) // RK(C)
    opcode(0, 1, OpArgK, OpArgK, IABC, "BAND    "), // R(A) := RK(B) & RK(C)
    opcode(0, 1, OpArgK, OpArgK, IABC, "BOR     "), // R(A) := RK(B) | RK(C)
    opcode(0, 1, OpArgK, OpArgK, IABC, "BXOR    "), // R(A) := RK(B) ~ RK(C)
    opcode(0, 1, OpArgK, OpArgK, IABC, "SHL     "), // R(A) := RK(B) << RK(C)
    opcode(0, 1, OpArgK, OpArgK, IABC, "SHR     "), // R(A) := RK(B) >> RK(C)
    opcode(0, 1, OpArgR, OpArgN, IABC, "UNM     "), // R(A) := -R(B)
    opcode(0, 1, OpArgR, OpArgN, IABC, "BNOT    "), // R(A) := ~R(B)
    opcode(0, 1, OpArgR, OpArgN, IABC, "NOT     "), // R(A) := not R(B)
    opcode(0, 1, OpArgR, OpArgN, IABC, "LEN     "), // R(A) := length of R(B)
    opcode(0, 1, OpArgR, OpArgR, IABC, "CONCAT  "), // R(A) := R(B).. ... ..R(C)
    opcode(0, 0, OpArgR, OpArgN, IAsBx, "JMP     "), // pc+=sBx; if (A) close all upvalues >= R(A - 1)
    opcode(1, 0, OpArgK, OpArgK, IABC, "EQ      "), // if ((RK(B) == RK(C)) ~= A) then pc++
    opcode(1, 0, OpArgK, OpArgK, IABC, "LT      "), // if ((RK(B) <  RK(C)) ~= A) then pc++
    opcode(1, 0, OpArgK, OpArgK, IABC, "LE      "), // if ((RK(B) <= RK(C)) ~= A) then pc++
    opcode(1, 0, OpArgN, OpArgU, IABC, "TEST    "), // if not (R(A) <=> C) then pc++
    opcode(1, 1, OpArgR, OpArgU, IABC, "TESTSET "), // if (R(B) <=> C) then R(A) := R(B) else pc++
    opcode(0, 1, OpArgU, OpArgU, IABC, "CALL    "), // R(A), ... ,R(A+C-2) := R(A)(R(A+1), ... ,R(A+B-1))
    opcode(0, 1, OpArgU, OpArgU, IABC, "TAILCALL"), // return R(A)(R(A+1), ... ,R(A+B-1))
    opcode(0, 0, OpArgU, OpArgN, IABC, "RETURN  "), // return R(A), ... ,R(A+B-2)
    opcode(0, 1, OpArgR, OpArgN, IAsBx, "FORLOOP "), // R(A)+=R(A+2); if R(A) <?= R(A+1) then { pc+=sBx; R(A+3)=R(A) }
    opcode(0, 1, OpArgR, OpArgN, IAsBx, "FORPREP "), // R(A)-=R(A+2); pc+=sBx
    opcode(0, 0, OpArgN, OpArgU, IABC, "TFORCALL"),  // R(A+3), ... ,R(A+2+C) := R(A)(R(A+1), R(A+2));
    opcode(0, 1, OpArgR, OpArgN, IAsBx, "TFORLOOP"), // if R(A+1) ~= nil then { R(A)=R(A+1); pc += sBx }
    opcode(0, 0, OpArgU, OpArgU, IABC, "SETLIST "),  // R(A)[(C-1)*FPF+i] := R(A+i), 1 <= i <= B
    opcode(0, 1, OpArgU, OpArgN, IABx, "CLOSURE "),  // R(A) := closure(KPROTO[Bx])
    opcode(0, 1, OpArgU, OpArgN, IABC, "VARARG  "),  // R(A), R(A+1), ..., R(A+B-2) = vararg
    opcode(0, 0, OpArgU, OpArgU, IAx, "EXTRAARG"),   // extra (larger) argument for previous opcode
];