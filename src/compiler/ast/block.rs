use super::{stat::Stat, exp::Exp};

#[derive(Clone, Debug)]
pub struct Block {
    pub last_line: i32,
    pub stats: Vec<Stat>,
    pub ret_exps: Vec<Exp>,
}





