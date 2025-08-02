use std::rc::Rc;
use super::block::Block;

#[derive(Clone, Debug)]
pub enum Exp {
    EmptyExp,
    NilExp { line: i32 },
    TrueExp { line: i32 },
    FalseExp { line: i32 },
    VarargExp { line: i32 },
    IntegerExp { line: i32, val: i64 },
    FloatExp { line: i32, val: f64 },
    StringExp { line: i32, str: String },
    NameExp { line: i32, str: String },
    UnopExp { 
        line: i32,
        op: i32,
        exp: Box<Exp>,
    },
    BinopExp {
        line: i32,
        op: i32,
        exp1: Box<Exp>,
        exp2: Box<Exp>,
    },
    ConcatExp {
        line: i32,  // line of last...
        exps: Vec<Exp>,
    },
    TableConstructorExp {
        line: i32,
        last_line: i32,
        key_exps: Vec<Exp>,
        val_exps: Vec<Exp>,
    },
    FuncDefExp {
        line: i32,
        last_line: i32,
        par_list: Vec<String>,
        is_vararg: bool,
        block: Rc<Block>,
    },
    ParensExp {
        exp: Box<Exp>,
    },
    TableAccessExp {
        last_line: i32,
        prefix_exp: Box<Exp>,
        key_exp: Box<Exp>,
    },
    FuncCallExp {
        line: i32,          // line of `(`
        last_line: i32,     // line of `)`
        prefix_exp: Box<Exp>,
        name_exp: Box<Exp>,     // specified for StringExp
        args: Vec<Exp>,
    }
}