use std::rc::Rc;
use crate::compiler::ast::exp::Exp::TrueExp;
use super::super::{ast::{block::Block, stat::Stat, exp::Exp}, lexer::{lexer::Lexer, token::*}};
use super::super::ast::exp::Exp::TableAccessExp;
use super::super::ast::stat::Stat::AssignStat;
use super::parse_exp::*;

// block := {stat}[retstat]
pub fn parse_block(lexer: &mut Lexer) -> Rc<Block> {
    Rc::new(
        Block {
            stats: parse_stats(lexer),
            ret_exps: parse_ret_exps(lexer),
            last_line: lexer.line(),
        }
    )
}

fn parse_stats(lexer: &mut Lexer) -> Vec<Stat> {
    let mut stats: Vec<Stat> = vec![];
    while !_is_return_or_block_end(lexer.look_ahead()) {
        let stat = parse_stat(lexer);
        if let Stat::EmptyStat = stat {} else {
            stats.push(stat);
        }
    }
    stats
}

fn _is_return_or_block_end(token_kind: Token) -> bool {
    match token_kind { 
        TOKEN_KW_RETURN | TOKEN_EOF | TOKEN_KW_END | TOKEN_KW_ELSE | TOKEN_KW_ELSEIF | TOKEN_KW_UNTIL => {
            true
        },
        _ => false
    }
}

// retstat ::= return [explist][`;`]
fn parse_ret_exps(lexer: &mut Lexer) -> Vec<Exp> {
    if lexer.look_ahead() != TOKEN_KW_RETURN {
        return vec![];
    }
    
    lexer.next_token();
    match lexer.look_ahead() { 
        TOKEN_EOF | TOKEN_KW_END | TOKEN_KW_ELSE |  TOKEN_KW_ELSEIF | TOKEN_KW_UNTIL => {
            vec![]
        },
        TOKEN_SEP_SEMI => {
            lexer.next_token();
            vec![]
        },
        _ => {
            let exps = parse_exp_list(lexer);
            if lexer.look_ahead() == TOKEN_SEP_SEMI {
                lexer.next_token();
            }
            exps
        },
    }
}

fn parse_stat(lexer: &mut Lexer) -> Stat {
    match lexer.look_ahead() {
        TOKEN_SEP_SEMI => {
            parse_empty_stat(lexer)
        },
        TOKEN_KW_BREAK => {
            parse_break_stat(lexer)
        },
        TOKEN_SEP_LABEL => {
            parse_label_stat(lexer)
        },
        TOKEN_KW_GOTO => {
            parse_goto_stat(lexer)
        },
        TOKEN_KW_DO => {
            parse_do_stat(lexer)
        },
        TOKEN_KW_WHILE => {
            parse_while_stat(lexer)
        },
        TOKEN_KW_REPEAT => {
            parse_repeat_stat(lexer)
        },
        TOKEN_KW_IF => {
            parse_if_stat(lexer)
        },
        TOKEN_KW_FOR => {
            parse_for_stat(lexer)
        },
        TOKEN_KW_FUNCTION => {
            parse_func_def_stat(lexer)
        },
        TOKEN_KW_LOCAL => {
            parse_local_assign_or_func_def_stat(lexer)
        },
        _ => {
            parse_assign_or_func_call_stat(lexer)
        }
    }
}

fn parse_empty_stat(lexer: &mut Lexer) -> Stat {
    lexer.next_token_of_kind(TOKEN_SEP_SEMI);       // `;`
    Stat::EmptyStat
}

fn parse_break_stat(lexer: &mut Lexer) -> Stat {
    lexer.next_token_of_kind(TOKEN_KW_BREAK);       // break
    Stat::BreakStat {
        line: lexer.line()
    }
}

fn parse_label_stat(lexer: &mut Lexer) -> Stat {
    lexer.next_token_of_kind(TOKEN_SEP_LABEL);      // `::`
    let (_, name) = lexer.next_identifier(); // Name
    lexer.next_token_of_kind(TOKEN_SEP_LABEL);      // `::`
    Stat::LabelStat {
        name,
    }
}

fn parse_goto_stat(lexer: &mut Lexer) -> Stat {
    lexer.next_token_of_kind(TOKEN_KW_GOTO);        // goto
    let (_, name) = lexer.next_identifier(); // Name
    Stat::GotoStat {
        name,
    }
}

fn parse_do_stat(lexer: &mut Lexer) -> Stat {
    lexer.next_token_of_kind(TOKEN_KW_DO);          // do
    let _block = parse_block(lexer);                // block
    lexer.next_token_of_kind(TOKEN_KW_END);         // end
    Stat::DoStat {
        block: _block,
    }
}

fn parse_while_stat(lexer: &mut Lexer) -> Stat {
    lexer.next_token_of_kind(TOKEN_KW_WHILE);       // while
    let _exp = parse_exp(lexer);                    // exp
    lexer.next_token_of_kind(TOKEN_KW_DO);          // do
    let _block = parse_block(lexer);                // block
    lexer.next_token_of_kind(TOKEN_KW_END);         // end
    Stat::WhileStat {
        exp: _exp,
        block: _block,
    }
}

fn parse_repeat_stat(lexer: &mut Lexer) -> Stat {
    lexer.next_token_of_kind(TOKEN_KW_REPEAT);      // repeat
    let _block = parse_block(lexer);                // block
    lexer.next_token_of_kind(TOKEN_KW_UNTIL);       // until
    let _exp = parse_exp(lexer);                    // exp
    Stat::RepeatStat {
        exp: _exp,
        block: _block,
    }
}

fn parse_if_stat(lexer: &mut Lexer) -> Stat {
    let mut exps = vec![];
    let mut blocks = vec![];
    
    lexer.next_token_of_kind(TOKEN_KW_IF);          // if
    exps.push(parse_exp(lexer));                    // exp
    lexer.next_token_of_kind(TOKEN_KW_THEN);        // then
    blocks.push(parse_block(lexer));                // block

    while lexer.look_ahead() == TOKEN_KW_ELSEIF {   // {
        lexer.next_token();                         // elseif
        exps.push(parse_exp(lexer));                // exp
        lexer.next_token_of_kind(TOKEN_KW_THEN);    // then
        blocks.push(parse_block(lexer));            // block
    }                                               // }
    
    // else block => elseif true then block
    if lexer.look_ahead() == TOKEN_KW_ELSE {        // [
        lexer.next_token();                         // else
        exps.push(TrueExp {
            line: lexer.line(),
        });                                         // 
        blocks.push(parse_block(lexer));            // block
    }                                               // ]
    
    lexer.next_token_of_kind(TOKEN_KW_END);         // end
    Stat::IfStat {
        exps,
        blocks,
    }
}

fn parse_for_stat(lexer: &mut Lexer) -> Stat {
    let (line_of_for, _) = lexer.next_token_of_kind(TOKEN_KW_FOR);
    let (_, name) = lexer.next_identifier();
    if lexer.look_ahead() == TOKEN_OP_ASSIGN {
        _finish_for_num_stat(lexer, line_of_for, name)
    } else {
        _finish_for_in_stat(lexer, name)
    }
}

fn _finish_for_num_stat(lexer: &mut Lexer, line_of_for: i32, var_name: String) -> Stat {
    lexer.next_token_of_kind(TOKEN_OP_ASSIGN);          // for name `=`
    let init_exp = parse_exp(lexer);               // exp
    lexer.next_token_of_kind(TOKEN_SEP_COMMA);          // `,`
    let limit_exp = parse_exp(lexer);              // exp
    
    let mut step_exp: Exp = if lexer.look_ahead() == TOKEN_SEP_COMMA {          // [
        lexer.next_token();                             // `,`
        parse_exp(lexer)                                // exp
    } else {                                            // ]
        Exp::IntegerExp {
            line: lexer.line(),
            val: 1,
        }
    };
    let (line_of_do, _) = lexer.next_token_of_kind(TOKEN_KW_DO);        // do
    let _block = parse_block(lexer);                                         // block
    lexer.next_token_of_kind(TOKEN_KW_END);                                  // end
    
    Stat::ForNumStat {
        line_of_for,
        line_of_do,
        var_name,
        init_exp,
        limit_exp,
        step_exp,
        block: _block,
    }
}

fn _finish_for_in_stat(lexer: &mut Lexer, name0: String) -> Stat {
    let name_list = _finish_name_list(lexer, name0);    // for name list
    lexer.next_token_of_kind(TOKEN_KW_IN);                           // in
    let exp_list = parse_exp_list(lexer);                 // exp list
    let (line_of_do, _) = lexer.next_token_of_kind(TOKEN_KW_DO);    // do
    let block = parse_block(lexer);                           // block
    lexer.next_token_of_kind(TOKEN_KW_END);                              // end
    Stat::ForInStat {
        line_of_do,
        name_list,
        exp_list,
        block,
    }
}

fn _finish_name_list(lexer: &mut Lexer, name0: String) -> Vec<String> {
    let mut names = vec![name0];                // name
    while lexer.look_ahead() == TOKEN_SEP_COMMA {       // {
        lexer.next_token();                             // `,`
        let (_, name) = lexer.next_identifier(); // name
        names.push(name);
    }                                                   // }
    names
}

fn parse_local_assign_or_func_def_stat(lexer: &mut Lexer) -> Stat {
    lexer.next_token_of_kind(TOKEN_KW_LOCAL);
    if lexer.look_ahead() == TOKEN_KW_FUNCTION {
        _finish_local_func_def_stat(lexer)
    } else {
        _finish_local_var_decl_stat(lexer)
    }
}

fn _finish_local_func_def_stat(lexer: &mut Lexer) -> Stat {
    lexer.next_token_of_kind(TOKEN_KW_FUNCTION);            // local function
    let (_, name) = lexer.next_identifier();         // name
    let fd_exp = parse_func_def_exp(lexer);                // func body
    Stat::LocalFuncDefStat {
        name,
        exp: Rc::new(fd_exp),
    }
}

fn _finish_local_var_decl_stat(lexer: &mut Lexer) -> Stat {
    let (_, name0) = lexer.next_identifier();
    let name_list = _finish_name_list(lexer, name0);
    let exp_list;
    if lexer.look_ahead() == TOKEN_OP_ASSIGN {
        lexer.next_token();
        exp_list = parse_exp_list(lexer);
    } else {
        exp_list = vec![];
    }
    let last_line = lexer.line();
    Stat::LocalVarDeclStat {
        last_line,
        name_list,
        exp_list,
    }
}

fn parse_assign_or_func_call_stat(lexer: &mut Lexer) -> Stat {
    let prefix_exp = parse_prefix_exp(lexer);
    if let Exp::FuncCallExp { .. } = prefix_exp {
        Stat::FuncCallStat(prefix_exp)
    } else {
        parse_assign_stat(lexer, prefix_exp)
    }
}

fn parse_assign_stat(lexer: &mut Lexer, var0: Exp) -> Stat {
    let var_list = _finish_var_list(lexer, var0);
    lexer.next_token_of_kind(TOKEN_OP_ASSIGN);
    let exp_list = parse_exp_list(lexer);
    let last_line = lexer.line();
    Stat::AssignStat {
        last_line,
        var_list,
        exp_list,
    }
}

fn _finish_var_list(lexer: &mut Lexer, var0: Exp) -> Vec<Exp> {
    let mut vars = vec![_check_var(lexer, var0)];
    while lexer.look_ahead() == TOKEN_SEP_COMMA {
        lexer.next_token();
        let exp = parse_prefix_exp(lexer);
        vars.push(_check_var(lexer, exp));
    }
    vars
}

fn _check_var(lexer: &mut Lexer, exp: Exp) -> Exp {
    match exp { 
        Exp::NameExp{ .. }  | TableAccessExp{ .. } => {
            exp
        },
        _ => {
            lexer.next_token_of_kind(-1);   // trigger error
            panic!("unreachable")
        },
    }
}

fn parse_func_def_stat(lexer: &mut Lexer) -> Stat {
    lexer.next_token_of_kind(TOKEN_KW_FUNCTION);
    let (fn_exp, has_colon) = _parse_fn_name(lexer);
    let mut fd_exp = parse_func_def_exp(lexer);
    let mut last_line_ = 0;
    if has_colon {
        if let Exp::FuncDefExp {ref line, last_line: _, ref mut par_list, is_vararg: _, block: _} = &mut fd_exp {
            par_list.push("".to_string());
            par_list.remove(0);
            par_list[0] = "self".to_string();
            last_line_ = line.clone();
        }
    }
    
    AssignStat {
        last_line: last_line_,
        var_list: vec![fn_exp],
        exp_list: vec![fd_exp],
    }
}

fn _parse_fn_name(lexer: &mut Lexer) -> (Exp, bool) {
    let (line, name) = lexer.next_identifier();
    let mut exp = Exp::NameExp {line, str: name};
    let mut has_colon = false;
    while lexer.look_ahead() == TOKEN_SEP_DOT {
        lexer.next_token();
        let (line, name) = lexer.next_identifier();
        let idx = Exp::StringExp {line, str: name};
        exp = TableAccessExp {
            last_line: line, 
            prefix_exp: Box::new(exp), 
            key_exp: Box::new(idx)
        };
    }
    if lexer.look_ahead() == TOKEN_SEP_COLON {
        lexer.next_token();
        let (line, name) = lexer.next_identifier();
        let idx = Exp::StringExp {line, str: name};
        exp = TableAccessExp {
            last_line: line,
            prefix_exp: Box::new(exp),
            key_exp: Box::new(idx)
        };
        has_colon = true;
    }
    (exp, has_colon)
}

#[cfg(test)]
mod tests{
    #[test]
    fn test0() {
        let a: i32;
        let b = 1;
        if b > 0 {
            a = 3;
        } else {
            a = 1;
        }
        assert_eq!(a, 3);
    }
}