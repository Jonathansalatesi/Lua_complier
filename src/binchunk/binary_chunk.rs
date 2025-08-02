use crate::state::lua_value::LuaValue;

pub const LUA_SIGNATURE: [u8; 4] = [0x1b, 0x4c, 0x75, 0x61];
pub const LUAC_VERSION: u8 = 0x53;
pub const LUAC_FORMAT: u8 = 0;
pub const LUAC_DATA: [u8; 6] = [0x19, 0x93, 0x0d, 0x0a, 0x1a, 0x0a];
pub const CINT_SIZE: u8 = 4;
pub const CSIZET_SIZE: u8 = 8;
pub const INSTRUCTION_SIZE: u8 = 4;
pub const LUA_INTEGER_SIZE: u8 = 8;
pub const LUA_NUMBER_SIZE: u8 = 8;
pub const LUAC_INT: i64 = 0x5678;
pub const LUAC_NUM: f64 = 370.5;

pub const TAG_NIL: u8 = 0x00;
pub const TAG_BOOLEAN: u8 = 0x01;
pub const TAG_NUMBER: u8 = 0x03;
pub const TAG_INTEGER: u8 = 0x13;
pub const TAG_SHORT_STR: u8 = 0x04;
pub const TAG_LONG_STR: u8 = 0x14;

struct BinaryChunk {
    header: Header,
    sizeUpvalues: u8,
    mainFunc: Prototype,
}

// Total 17 + 16 = 33
struct Header {
    signature: [u8; 4],
    version: u8,
    format: u8,
    luacData: [u8; 6],
    cintSize: u8,
    sizetSize: u8,
    instructionSize: u8,
    luaIntegerSize: u8,
    luaNumberSize: u8,
    luacInt: i64,
    luacNum: f64,
}

// function prototype
#[derive(Debug, Clone)]
pub struct Prototype {
    pub source: Option<String>, // debug
    pub lineDefined: u32,
    pub lastLineDefined: u32,
    pub numParams: u8,
    pub isVararg: u8,
    pub maxStackSize: u8,
    pub code: Vec<u32>,
    pub constants: Vec<LuaValue>,
    pub upvalues: Vec<Upvalue>,
    pub protos: Vec<Prototype>,
    pub lineInfo: Vec<u32>,        // debug
    pub locVars: Vec<LocVar>,      // debug
    pub upvalueNames: Vec<String>, // debug
}

impl Prototype {
    pub fn FakeProto() -> Self {
        Prototype {
            source: None,
            lineDefined: 0,
            lastLineDefined: 0,
            numParams: 0,
            isVararg: 0,
            maxStackSize: 0,
            code: vec![],
            constants: vec![],
            upvalues: vec![],
            protos: vec![],
            lineInfo: vec![],        // debug
            locVars: vec![],      // debug
            upvalueNames: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Upvalue {
    pub instack: u8,
    pub idx: u8,
}

#[derive(Debug, Clone)]
pub struct LocVar {
    pub varName: String,
    pub startPC: u32,
    pub endPC: u32,
}