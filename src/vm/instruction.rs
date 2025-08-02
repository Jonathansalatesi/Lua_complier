use super::{inst_call::*, inst_for::*, inst_load::*, inst_misc::*, inst_operators::*, inst_table::*, inst_upvalue::{self, getTabUp, getUpVal, setTabUp, setUpVal}, opcodes::*};
use crate::api::{lua_vm::LuaVM, consts::*};

pub const MAXARG_Bx: i32 = (1 << 18) - 1;
pub const MAXARG_sBx: i32 = MAXARG_Bx >> 1;

pub struct Instruction {
    i: u32,
}

impl Instruction {
    pub fn new(i: u32) -> Self {
        Self { i: i }
    }

    pub fn Opcode(&self) -> i32 {
        (self.i & 0x3f) as i32
    }

    pub fn ABC(&self) -> (i32, i32, i32) {
        let a = ((self.i >> 6) & 0xff) as i32;
        let c = ((self.i >> 14) & 0x1ff) as i32;
        let b = ((self.i >> 23) & 0x1ff) as i32;
        (a, b, c)
    } 

    pub fn ABx(&self) -> (i32, i32) {
        let a = ((self.i >> 6) & 0xff) as i32;
        let bx = (self.i >> 14) as i32;
        (a, bx)
    }

    pub fn AsBx(&self) -> (i32, i32) {
        let (a, bx) = self.ABx();
        (a, bx - MAXARG_sBx)
    }

    pub fn Ax(&self) -> i32 {
        (self.i >> 6) as i32
    }

    pub fn OpName(&self) -> &'static str {
        OPCODES[self.Opcode() as usize].name
    }

    pub fn OpMode(&self) -> Mode {
        OPCODES[self.Opcode() as usize].opMode.clone()
    }

    pub fn BMode(&self) -> OpArg {
        OPCODES[self.Opcode() as usize].argBMode.clone()
    }

    pub fn CMode(&self) -> OpArg {
        OPCODES[self.Opcode() as usize].argCMode.clone()
    }

    pub fn Execute(&self, vm: &mut dyn LuaVM) {
        match self.Opcode() as u8 {
            OP_MOVE => move_(self, vm),
            OP_LOADK => loadK(self, vm),
            OP_LOADKX => loadKx(self, vm),
            OP_LOADBOOL => loadBool(self, vm),
            OP_LOADNIL => loadNil(self, vm),
            OP_GETUPVAL => getUpVal(self, vm),
            OP_GETTABUP => getTabUp(self, vm),
            OP_GETTABLE => getTable(self, vm),
            OP_SETTABUP => setTabUp(self, vm),
            OP_SETUPVAL => setUpVal(self, vm),
            OP_SETTABLE => setTable(self, vm),
            OP_NEWTABLE => newTable(self, vm),
            OP_SELF => _self(self, vm),
            OP_ADD => add(self, vm),
            OP_SUB => sub(self, vm),
            OP_MUL => mul(self, vm),
            OP_MOD => _mod(self, vm),
            OP_POW => pow(self, vm),
            OP_DIV => div(self, vm),
            OP_IDIV => idiv(self, vm),
            OP_BAND => band(self, vm),
            OP_BOR => bor(self, vm),
            OP_BXOR => bxor(self, vm),
            OP_SHL => shl(self, vm),
            OP_SHR => shr(self, vm),
            OP_UNM => unm(self, vm),
            OP_BNOT => bnot(self, vm),
            OP_NOT => not(self, vm),
            OP_LEN => _len(self, vm),
            OP_CONCAT => concat(self, vm),
            OP_JMP => jmp(self, vm),
            OP_EQ => eq(self, vm),
            OP_LT => lt(self, vm),
            OP_LE => le(self, vm),
            OP_TEST => test(self, vm),
            OP_TESTSET => testSet(self, vm),
            OP_CALL => call(self, vm),
            OP_TAILCALL => tailcall(self, vm),
            OP_RETURN => _return(self, vm),
            OP_FORLOOP => for_loop(self, vm),
            OP_FORPREP => for_prep(self, vm),
            OP_TFORCALL => tForCall(self, vm),
            OP_TFORLOOP => tForLoop(self, vm),
            OP_SETLIST => setList(self, vm),
            OP_CLOSURE => closure(self, vm),
            OP_VARARG => vararg(self, vm),
            // OP_EXTRAARG => (),
            _ => {
                dbg!(self.OpName());
                unimplemented!()
            }
        }
    }
}