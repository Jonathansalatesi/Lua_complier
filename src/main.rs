use std::{env, fs::File, io::prelude::*, io};

use api::consts::LUA_TNIL;
use state::lua_state::LuaState;
use crate::binchunk::binary_chunk::Prototype;
use crate::api::lua_state::LuaAPI;
use crate::compiler::{codegen::compile, disassembly};

mod api;
mod binchunk;
mod compiler;
mod number;
mod state;
mod vm;

// ================================================================
// Function for disassembling.
// ================================================================
fn disassemble_file(chunk: Vec<u8>, chunk_name: &str) {
    let proto: Prototype = if state::lua_state::is_binary_chunk(&chunk) {
        binchunk::undump(chunk)
    } else {
        let s_chunk: String = chunk.iter().map(|s|{*s as char}).collect();
        compile(s_chunk, chunk_name.to_owned())
    };
    disassembly(&proto);
}

// ================================================================
// Function for parsing cmd codes and entry point of program
// ================================================================
fn main() -> io::Result<()> {
    let mut filename = "".to_owned();
    if env::args().count() == 2 {
        filename = env::args().nth(1).unwrap();
        if filename.starts_with("-h") || filename.starts_with("--help") {
            print!("Usage: lua <filename>\n");
            print!("       lua [Optional] <filename>\n");
            print!("Options:\n");
            print!("\t-h or --help\t\thelps\n");
            print!("\t-l or --asm\t\tdisassemble programs\n");
            print!("\t-v or --version\t\tshow version of complier\n");
            return Ok(());
        } else if filename.starts_with("-v") || filename.starts_with("--version") {
            print!("Lua_complier v0.1.0\n");
            return Ok(());
        }
    } else if env::args().count() == 3 {
        let options = env::args().nth(1).unwrap();
        filename = env::args().nth(2).unwrap();
        if options.starts_with("-l") || options.starts_with("--asm") {
            let mut file = File::open(&filename)?;
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;
            disassemble_file(data, &filename);
            return Ok(());
        } else {
            panic!("Invalid cmd options.\n");
        }
    }
    
    if filename.len() > 0 {
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
    }
    
    // if env::args().count() >= 0 {
    //     // let filename = env::args().nth(1).unwrap();
    //     // let filename = String::from(r"D:\usr\lua-5.3.4_Win64_bin\luac.out");
    //     // let filename = String::from(r"D:\usr\orginal_code_analysis\luago-book-master\code\lua\ch02\hello_world.lua");
    //     // let filename = String::from(r"D:\usr\orginal_code_analysis\luago-book-master\code\lua\ch10\factorial.lua");
    //     let filename = String::from(r"D:\usr\orginal_code_analysis\luago-book-master\code\lua\ch12\examples.lua");
    //     // let filename = String::from(r"D:\usr\orginal_code_analysis\luago-book-master\code\lua\ch06\sum.lua");
    //     let mut file = File::open(&filename)?;
    // 
    //     let mut data = Vec::new();
    //     file.read_to_end(&mut data)?;
    // 
    //     let mut ls = LuaState::new();
    //     ls.Register("print", __print__);
    //     ls.Register("getmetatable", __getMetatable__);
    //     ls.Register("setmetatable", __setMetatable__);
    //     ls.Register("next", __next__);
    //     ls.Register("pairs", __pairs__);
    //     ls.Register("ipairs", __ipairs__);
    //     ls.Load(data, &filename, "bt");
    //     ls.Call(0, 0);
    //     // disassemble_file(data, &filename);
    // }
    Ok(())
}

// ================================================================
// Following are the lua registered functions.
// ================================================================
fn __print__(ls: &mut LuaState) -> i32 {
    let nArgs = ls.GetTop();
    for i in 1..=nArgs {
        if ls.IsBoolean(i) {
            print!("{}", ls.ToBoolean(i));
        } else if ls.IsString(i) {
            print!("{}", ls.ToString(i));
        }else if ls.IsNil(i) {
            print!("nil");
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