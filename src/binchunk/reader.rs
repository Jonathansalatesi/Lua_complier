use std::env::set_current_dir;

use crate::{binchunk::binary_chunk::*, state::lua_value::LuaValue};

pub struct Reader {
    data: Vec<u8>,
}

impl Reader {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data: data }
    }

    pub fn readByte(&mut self) -> u8 {
        self.data.remove(0)
    }

    pub fn readUint32(&mut self) -> u32 {
        let a0 = self.readByte() as u32;
        let a1 = self.readByte() as u32;
        let a2 = self.readByte() as u32;
        let a3 = self.readByte() as u32;
        (a3 << 24) | (a2 << 16) | (a1 << 8) | a0
    }

    pub fn readUint64(&mut self) -> u64 {
        let a0 = self.readUint32() as u64;
        let a1 = self.readUint32() as u64;
        (a1 << 32) | a0
    }

    pub fn readLuaInteger(&mut self) -> i64 {
        self.readUint64() as i64
    }

    pub fn readLuaNumber(&mut self) -> f64 {
        use std::f64;
        f64::from_bits(self.readUint64())
    }

    pub fn readString(&mut self) -> String {
        let mut length = self.readByte() as usize;
        if length == 0 {
            return String::from("");
        }
        if length == 0xFF {
            length = self.readUint64() as usize;
        }
        let bytes = self.readBytes(length - 1);
        if let Ok(res) = String::from_utf8(bytes) {
            res
        } else {
            String::from("")
        }
    }

    pub fn readBytes(&mut self, n: usize) -> Vec<u8> {
        let mut vec = vec![];
        for _ in 0..n {
            vec.push(self.readByte());
        }
        vec
    }

    pub fn checkHeader(&mut self) {
        assert_eq!(self.readBytes(4), LUA_SIGNATURE, "not a precompiled chunk!");
        assert_eq!(self.readByte(), LUAC_VERSION, "version mismatch!");
        assert_eq!(self.readByte(), LUAC_FORMAT, "format mismatch!");
        assert_eq!(self.readBytes(6), LUAC_DATA, "corrupted!");
        assert_eq!(self.readByte(), CINT_SIZE, "int size mismatch!");
        assert_eq!(self.readByte(), CSIZET_SIZE, "size_t size mismatch!");
        assert_eq!(self.readByte(), INSTRUCTION_SIZE, "instruction size mismatch!");
        assert_eq!(self.readByte(), LUA_INTEGER_SIZE, "lua_Integer size mismatch!");
        assert_eq!(self.readByte(), LUA_NUMBER_SIZE, "lua_Number size mismatch!");
        assert_eq!(self.readLuaInteger(), LUAC_INT, "endianness mismatch!");
        assert_eq!(self.readLuaNumber(), LUAC_NUM, "float format mismatch!");
    }

    pub fn readProto(&mut self, parentSource: String) -> Prototype {
        let mut source = self.readString();
        if source == "" {
            source = parentSource;
        }

        Prototype {
            source: Some(source.clone()),
            lineDefined: self.readUint32(),
            lastLineDefined: self.readUint32(),
            numParams: self.readByte(),
            isVararg: self.readByte(),
            maxStackSize: self.readByte(),
            code: self.readCode(),
            constants: self.readConstants(),
            upvalues: self.readUpvalues(),
            protos: self.readProtos(source.clone()),
            lineInfo: self.readLineInfo(),
            locVars: self.readLocVars(),
            upvalueNames: self.readUpvalueNames(),
        }
    }

    fn readCode(&mut self) -> Vec<u32> {
        let num_codes = self.readUint32() as usize;
        let mut code = Vec::<u32>::with_capacity(num_codes);
        for _ in 0..num_codes {
            code.push(self.readUint32());
        }
        code
    }

    fn readConstant(&mut self) -> LuaValue {
        match self.readByte() {
            TAG_NIL => LuaValue::Nil,
            TAG_BOOLEAN => LuaValue::Bool(self.readByte() != 0),
            TAG_INTEGER => LuaValue::Integer(self.readLuaInteger()),
            TAG_NUMBER => LuaValue::Number(self.readLuaNumber()),
            TAG_SHORT_STR => LuaValue::Str(self.readString()),
            TAG_LONG_STR => LuaValue::Str(self.readString()),
            _ => panic!("corrupted!"),
        }
    }

    fn readConstants(&mut self) -> Vec<LuaValue> {
        let num_constants = self.readUint32() as usize;
        let mut res = Vec::<LuaValue>::with_capacity(num_constants);
        for _ in 0..num_constants {
            res.push(self.readConstant());
        }
        res
    }

    fn readUpvalues(&mut self) -> Vec<Upvalue> {
        let num_upvalues = self.readUint32() as usize;
        let mut upvalues = Vec::<Upvalue>::with_capacity(num_upvalues);
        for _ in 0..num_upvalues {
            upvalues.push(Upvalue {
                instack: self.readByte(),
                idx: self.readByte(),
            });
        }
        upvalues
    }

    fn readProtos(&mut self, parentSource: String) -> Vec<Prototype> {
        let num_protos = self.readUint32() as usize;
        let mut protos = Vec::<Prototype>::with_capacity(num_protos);
        for _ in 0..num_protos {
            protos.push(self.readProto(parentSource.clone()));
        }
        protos
    }

    fn readLineInfo(&mut self) -> Vec<u32> {
        let num_lineinfos = self.readUint32() as usize;
        let mut lineInfo = Vec::<u32>::with_capacity(num_lineinfos);
        for _ in 0..num_lineinfos {
            lineInfo.push(self.readUint32());
        }
        lineInfo
    }

    fn readLocVars(&mut self) -> Vec<LocVar> {
        let num_locvars = self.readUint32() as usize;
        let mut locVars = Vec::<LocVar>::with_capacity(num_locvars);
        for _ in 0..num_locvars {
            locVars.push(LocVar {
                varName: self.readString(),
                startPC: self.readUint32(),
                endPC: self.readUint32(),
            });
        }
        locVars
    }

    fn readUpvalueNames(&mut self) -> Vec<String> {
        let num_names = self.readUint32() as usize;
        let mut names = Vec::<String>::with_capacity(num_names);
        for _ in 0..num_names {
            names.push(self.readString());
        }
        names
    }

}