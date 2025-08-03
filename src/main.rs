use std::{env, fs::File, io::prelude::*, io};

use api::consts::LUA_TNIL;
use state::lua_state::LuaState;
use crate::binchunk::binary_chunk::Prototype;
use crate::api::{lua_vm::LuaVM, lua_state::LuaAPI};
use crate::compiler::{codegen::compile, disassembly};
use crate::vm::instruction::Instruction;

mod api;
mod binchunk;
mod compiler;
mod number;
mod state;
mod vm;

fn disasm(chunk: Vec<u8>, chunk_name: &str) {
    let s_chunk: String = chunk.iter().map(|s|{*s as char}).collect();
    let proto = compile(s_chunk, chunk_name.to_owned());
    disassembly(&proto);
}

fn main() -> io::Result<()> {
    // source: D:\usr\lua-5.3.4_Win64_bin\luac.out
    if env::args().count() >= 0 {
        // let filename = env::args().nth(1).unwrap();
        // let filename = String::from(r"D:\usr\lua-5.3.4_Win64_bin\luac.out");
        // let filename = String::from(r"D:\usr\orginal_code_analysis\luago-book-master\code\lua\ch02\hello_world.lua");
        let filename = String::from(r"D:\usr\orginal_code_analysis\luago-book-master\code\lua\ch10\factorial.lua");
        // let filename = String::from(r"D:\usr\orginal_code_analysis\luago-book-master\code\lua\ch06\sum.lua");
        let mut file = File::open(&filename)?;

        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        let mut ls = LuaState::new();
        ls.Register("print", __print__);
        ls.Register("getmetatable", __getMetatable__);
        ls.Register("setmetatable", __setMetatable__);
        ls.Register("next", __next__);
        ls.Register("pairs", __pairs__);
        ls.Register("ipairs", __ipairs__);
        ls.Load(data, &filename, "bt");
        ls.Call(0, 0);
        // disasm(data, &filename);
    }
    Ok(())
}

fn __print__(ls: &mut LuaState) -> i32 {
    let nArgs = ls.GetTop();
    for i in 1..=nArgs {
        if ls.IsBoolean(i) {
            print!("{}", ls.ToBoolean(i));
        } else if ls.IsString(i) {
            print!("{}", ls.ToString(i));
        } else {
            print!("({})", ls.TypeName(ls.Type(i)));
        }
        if i < nArgs {
            print!("\t");
        }
    }
    println!("");
    0
}

fn __getMetatable__(ls: &mut LuaState) -> i32 {
    if !ls.GetMetatable(1) {
        ls.PushNil();
    }
    1
}

fn __setMetatable__(ls: &mut LuaState) -> i32 {
    ls.SetMetatable(1);
    1
}

fn __next__(ls: &mut LuaState) -> i32 {
    ls.SetTop(2);       // 若参数2不存在则设置为nil
    if ls.Next(1) {
        return 2;
    } else {
        ls.PushNil();
        return 1;
    }
}

fn __pairs__(ls: &mut LuaState) -> i32 {
    ls.PushRustFunction(__next__);          // will return generator
    ls.PushValue(1);                   // state
    ls.PushNil();
    3
}

fn __ipairs__(ls: &mut LuaState) -> i32 {
    ls.PushRustFunction(__iPairsAux);       // iteration function
    ls.PushValue(1);                    // state
    ls.PushInteger(0);                      // initial value
    3
}

fn __iPairsAux(ls: &mut LuaState) -> i32 {
    let i = ls.ToInteger(2) + 1;
    ls.PushInteger(i);
    if ls.GetI(1, i) == LUA_TNIL {
        return 1;
    } else {
        return 2;
    }
}