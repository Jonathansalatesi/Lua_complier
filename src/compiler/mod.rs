pub mod ast;
pub mod lexer;
pub mod parser;
pub mod codegen;

use crate::state::lua_value::LuaValue;
use crate::binchunk::binary_chunk::Prototype;
use crate::vm::{instruction::Instruction, opcodes::{Mode, OpArg}};

pub fn disassembly(f: &Prototype) {
    list(f);
}

fn list(f: &Prototype) {
    print_header(f);
    print_code(f);
    print_detail(f);
    for p in &(f.protos) {
        list(p);
    }
}

fn print_header(f: &Prototype) {
    let func_type = if f.lineDefined > 0 { "function" } else { "main" };
    let vararg_flag = if f.isVararg > 0 { "+" } else { "" };
    let source = f.source.as_ref().map(|x| x.as_str()).unwrap_or("");

    print!("\n{}", func_type);
    print!(" <{}:{},{}>", source, f.lineDefined, f.lastLineDefined);
    print!(" ({} instructions)\n", f.code.len());
    print!("{}{} params", f.numParams, vararg_flag);
    print!(", {} slots", f.maxStackSize);
    print!(", {} upvalues", f.upvalues.len());
    print!(", {} locals", f.locVars.len());
    print!(", {} constants", f.constants.len());
    print!(", {} functions\n", f.protos.len());
}

fn print_code(f: &Prototype) {
    for pc in 0..f.code.len() {
        let line = f.lineInfo.get(pc).map(|n| n.to_string()).unwrap_or(String::from("-"));
        let instr = Instruction::new(f.code[pc]);
        print!("\t{}\t[{}]\t{} \t", pc + 1, line, instr.OpName());
        print_operands(instr);
        println!("");
    }
}

fn print_operands(i: Instruction) {
    match i.OpMode() {
        Mode::IABC => print_abc(i),
        Mode::IABx => print_abx(i),
        Mode::IAsBx => print_asbx(i),
        Mode::IAx => print_ax(i),
    }
}

fn print_abc(i: Instruction) {
    let (a, b, c) = i.ABC();
    print!("{}", a);
    match i.BMode() {
        OpArg::OpArgN => {},
        _ => {
            if b > 0xFF {
                print!(" {}", -1 - (b & 0xFF));
            } else {
                print!(" {}", b);
            }
        }
    };

    match i.CMode() {
        OpArg::OpArgN => {},
        _ => {
            if c > 0xFF {
                print!(" {}", -1 - (c & 0xFF));
            } else {
                print!(" {}", c);
            }
        }
    };
}

fn print_abx(i: Instruction) {
    let (a, bx) = i.ABx();
    print!("{}", a);
    match i.BMode() {
        OpArg::OpArgK => {
            print!(" {}", -1 - bx);
        },
        OpArg::OpArgU => {
            print!(" {}", bx);
        },
        _ => {},
    };
}

fn print_asbx(i: Instruction) {
    let (a, sbx) = i.AsBx();
    print!("{} {}", a, sbx);
}

fn print_ax(i: Instruction) {
    let ax = i.Ax();
    print!("{}", -1 - ax);
}

fn print_detail(f: &Prototype) {
    print_consts(f);
    print_locals(f);
    print_upvals(f)
}

fn print_consts(f: &Prototype) {
    let n = f.constants.len();
    println!("constants ({}):", n);
    for i in 0..n {
        print_const(i + 1, &f.constants[i]);
    }
}

fn print_const(n: usize, k: &LuaValue) {
    match k {
        LuaValue::Nil => println!("\t{}\tnil", n),
        LuaValue::Bool(b) => println!("\t{}\t{}", n, b),
        LuaValue::Number(x) => println!("\t{}\t{}", n, x),
        LuaValue::Integer(i) => println!("\t{}\t{}", n, i),
        LuaValue::Str(s) => println!("\t{}\t{:?}", n, s),
        LuaValue::Table(table) => println!("\t{}\t{:#?}", n, *(table.borrow())),
        LuaValue::Function(f) => println!("\t{}\t{:#?}", n, **f),
    }
}

fn print_locals(f: &Prototype) {
    let n = f.locVars.len();
    println!("locals ({}):", n);
    for i in 0..n {
        let var = &f.locVars[i];
        println!("\t{}\t{}\t{}\t{}", i, var.varName, var.startPC + 1, var.endPC + 1);
    }
}

fn print_upvals(f: &Prototype) {
    let n = f.upvalues.len();
    println!("upvalues ({}):", n);
    for i in 0..n {
        let upval = &f.upvalues[i];
        let name = f.upvalueNames.get(i).map(|x| x.as_str()).unwrap_or("");
        println!("\t{}\t{}\t{}\t{}", i, name, upval.instack, upval.idx);
    }
}