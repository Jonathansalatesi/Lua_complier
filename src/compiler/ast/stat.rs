use std::rc::Rc;
use super::{block::Block, exp::Exp};

#[derive(Clone, Debug)]
pub enum Stat {
    EmptyStat,
    // break
    BreakStat { 
        line: i32,
    },
    // `::` Name `::`
    LabelStat {
        name: String,
    },
    // goto Name
    GotoStat {
        name: String,
    },
    // do block end
    DoStat {
        block: Rc<Block>,
    },
    // function call
    FuncCallStat(Exp),      // specified for FuncCallExp
    
    WhileStat {
        exp: Exp,
        block: Rc<Block>,
    },
    RepeatStat {
        block: Rc<Block>,
        exp: Exp,
    },
    IfStat {
        exps: Vec<Exp>,
        blocks: Vec<Rc<Block>>,
    },
    ForNumStat {
        line_of_for: i32,
        line_of_do: i32,
        var_name: String,
        init_exp: Exp,
        limit_exp: Exp,
        step_exp: Exp,
        block: Rc<Block>,
    },
    ForInStat { 
        line_of_do: i32,
        name_list: Vec<String>,
        exp_list: Vec<Exp>,
        block: Rc<Block>,
    },
    LocalVarDeclStat {
        last_line: i32,
        name_list: Vec<String>,
        exp_list: Vec<Exp>,
    },
    AssignStat {
        last_line: i32,
        var_list: Vec<Exp>,
        exp_list: Vec<Exp>,
    },
    LocalFuncDefStat {
        name: String,
        exp: Rc<Exp>,
    },
}