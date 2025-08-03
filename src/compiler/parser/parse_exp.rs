use super::super::ast::exp::Exp;
use super::super::ast::exp::Exp::*;
use super::super::lexer::lexer::Lexer;
use super::super::lexer::token::*;
use super::parse_block::*;

pub fn parse_exp_list(lexer: &mut Lexer) -> Vec<Exp> {
    let mut exps: Vec<Exp> = vec![];
    exps.push(parse_exp(lexer));                        // exp
    while lexer.look_ahead() == TOKEN_SEP_COMMA {       // {
        lexer.next_token();                             // `,`
        exps.push(parse_exp(lexer));                    // exp
    }                                                   // }
    exps
}

// exp ::= exp12
pub fn parse_exp(lexer: &mut Lexer) -> Exp {
    parse_exp12(lexer)
}

// exp12 ::= exp11 {or exp11}
fn parse_exp12(lexer: &mut Lexer) -> Exp {
    let mut exp = parse_exp11(lexer);
    while lexer.look_ahead() == TOKEN_OP_OR {
        let (line, op, _) = lexer.next_token();
        exp = BinopExp {
            line,
            op,
            exp1: Box::new(exp),
            exp2: Box::new(parse_exp11(lexer)),
        };
    }
    exp
}

// exp11 ::= exp10 {and exp10}
fn parse_exp11(lexer: &mut Lexer) -> Exp {
    let mut exp = parse_exp10(lexer);
    while lexer.look_ahead() == TOKEN_OP_AND {
        let (line, op, _) = lexer.next_token();
        exp = BinopExp {
            line,
            op,
            exp1: Box::new(exp),
            exp2: Box::new(parse_exp10(lexer)),
        };
    }
    exp
}

// exp10 ::= exp9 {(‘<’ | ‘>’ | ‘<=’ | ‘>=’ | ‘~=’ | ‘==’) exp9}
fn parse_exp10(lexer: &mut Lexer) -> Exp {
    let mut exp = parse_exp9(lexer);
    loop {
        match lexer.look_ahead() {
            TOKEN_OP_LT | TOKEN_OP_GT | TOKEN_OP_NE | TOKEN_OP_LE | TOKEN_OP_GE | TOKEN_OP_EQ => {
                let (line, op, _) = lexer.next_token();
                exp = BinopExp {
                    line,
                    op,
                    exp1: Box::new(exp),
                    exp2: Box::new(parse_exp9(lexer)),
                };
            },
            _ => {
                return exp;
            }
        }
    }
}

// exp9  ::= exp8 {‘|’ exp8}
fn parse_exp9(lexer: &mut Lexer) -> Exp {
    let mut exp = parse_exp8(lexer);
    while lexer.look_ahead() == TOKEN_OP_BOR {
        let (line, op, _) = lexer.next_token();
        exp = BinopExp {
            line,
            op,
            exp1: Box::new(exp),
            exp2: Box::new(parse_exp8(lexer)),
        };
    }
    exp
}

// exp8  ::= exp7 {‘~’ exp7}
fn parse_exp8(lexer: &mut Lexer) -> Exp {
    let mut exp = parse_exp7(lexer);
    while lexer.look_ahead() == TOKEN_OP_BXOR {
        let (line, op, _) = lexer.next_token();
        exp = BinopExp {
            line,
            op,
            exp1: Box::new(exp),
            exp2: Box::new(parse_exp7(lexer)),
        };
    }
    exp
}

// exp7  ::= exp6 {‘&’ exp6}
fn parse_exp7(lexer: &mut Lexer) -> Exp {
    let mut exp = parse_exp6(lexer);
    while lexer.look_ahead() == TOKEN_OP_BAND {
        let (line, op, _) = lexer.next_token();
        exp = BinopExp {
            line,
            op,
            exp1: Box::new(exp),
            exp2: Box::new(parse_exp6(lexer)),
        };
    }
    exp
}

// exp6  ::= exp5 {(‘<<’ | ‘>>’) exp5}
fn parse_exp6(lexer: &mut Lexer) -> Exp {
    let mut exp = parse_exp5(lexer);
    while lexer.look_ahead() == TOKEN_OP_SHL || lexer.look_ahead() == TOKEN_OP_SHR  {
        let (line, op, _) = lexer.next_token();
        exp = BinopExp {
            line,
            op,
            exp1: Box::new(exp),
            exp2: Box::new(parse_exp5(lexer)),
        };
    }
    exp
}

// exp5  ::= exp4 {‘..’ exp4}
fn parse_exp5(lexer: &mut Lexer) -> Exp {
    let mut exp = parse_exp4(lexer);
    if lexer.look_ahead() != TOKEN_OP_CONCAT {
        return exp;
    }
    
    let mut line = 0;
    let mut exps = vec![exp];
    while lexer.look_ahead() == TOKEN_OP_CONCAT {
        (line, _, _) = lexer.next_token();
        exps.push(parse_exp4(lexer));
    }
    ConcatExp {
        line,
        exps
    }
}

// exp4  ::= exp3 {(‘+’ | ‘-’) exp3}
fn parse_exp4(lexer: &mut Lexer) -> Exp {
    let mut exp = parse_exp3(lexer);
    while lexer.look_ahead() == TOKEN_OP_ADD || lexer.look_ahead() == TOKEN_OP_SUB  {
        let (line, op, _) = lexer.next_token();
        exp = BinopExp {
            line,
            op,
            exp1: Box::new(exp),
            exp2: Box::new(parse_exp3(lexer)),
        };
    }
    exp
}

// exp3  ::= exp2 {(‘*’ | ‘/’ | ‘//’ | ‘%’) exp2}
fn parse_exp3(lexer: &mut Lexer) -> Exp {
    let mut exp = parse_exp2(lexer);
    loop {
        match lexer.look_ahead() {
            TOKEN_OP_MUL | TOKEN_OP_MOD | TOKEN_OP_DIV | TOKEN_OP_IDIV  => {
                let (line, op, _) = lexer.next_token();
                exp = BinopExp {
                    line,
                    op,
                    exp1: Box::new(exp),
                    exp2: Box::new(parse_exp2(lexer)),
                };
            },
            _ => {
                return exp;
            }
        }
    }
}

// exp2  ::= {(‘not’ | ‘#’ | ‘-’ | ‘~’)} exp1
fn parse_exp2(lexer: &mut Lexer) -> Exp {   // not | # | - | ~
    match lexer.look_ahead() { 
        TOKEN_OP_UNM | TOKEN_OP_BNOT | TOKEN_OP_LEN | TOKEN_OP_NOT => {
            let (line, op, _) = lexer.next_token();
            UnopExp {
                line, 
                op,
                exp: Box::new(parse_exp2(lexer)),
            }
        },
        _ => {
            parse_exp1(lexer)
        },
    }
}

// exp1  ::= exp0 {‘^’ exp2}
fn parse_exp1(lexer: &mut Lexer) -> Exp {   // exp0 ^ exp2
    let mut exp = parse_exp0(lexer);
    if lexer.look_ahead() == TOKEN_OP_POW {
        let (line, op, _) = lexer.next_token();
        exp = BinopExp {
            line,
            op,
            exp1: Box::new(exp),
            exp2: Box::new(parse_exp2(lexer)),
        }
    }
    exp
}

// exp0  ::= nil | false | true | Numeral | LiteralString
// 		| ‘...’ | functiondef | prefixexp | tableconstructor
fn parse_exp0(lexer: &mut Lexer) -> Exp {
    match lexer.look_ahead() {
        TOKEN_VARARG => { // ...
            let (line, _, _) = lexer.next_token();
            VarargExp { 
                line
            }
        },
        TOKEN_KW_NIL => { // nil
            let (line, _, _) = lexer.next_token();
            NilExp { 
                line 
            }
        },
        TOKEN_KW_TRUE => { // true
            let (line, _, _) = lexer.next_token();
            TrueExp { 
                line 
            }
        },
        TOKEN_KW_FALSE => { // false
            let (line, _, _) = lexer.next_token();
            FalseExp { 
                line 
            }
        },
        TOKEN_STRING => { // LiteralString
            let (line, _, token) = lexer.next_token();
            StringExp { 
                line, 
                str: token 
            }
        },
        TOKEN_NUMBER => { // Numeral
            parse_number_exp(lexer)
        },
        TOKEN_SEP_LCURLY => { // tableconstructor
            parse_table_constructor_exp(lexer)
        },
        TOKEN_KW_FUNCTION => { // functiondef
            lexer.next_token();
            parse_func_def_exp(lexer)
        },
        _ => { // prefixexp
            parse_prefix_exp(lexer)
        }
    }
}

fn parse_number_exp(lexer: &mut Lexer) -> Exp {
    let (line, _, token) = lexer.next_token();
    if let Ok(i_val) = (&token).parse::<i64>() {        // only decimal!
        IntegerExp {
            line,
            val: i_val,
        }
    } else if let Ok(i_val) = i64::from_str_radix(&token, 2){
        IntegerExp {
            line,
            val: i_val,
        }
    } else if let Ok(i_val) = i64::from_str_radix(&token, 16){
        IntegerExp {
            line,
            val: i_val,
        }
    } else if  let Ok(f_val) = (&token).parse::<f64>() {
        FloatExp {
            line,
            val: f_val,
        }
    } else {
        panic!("not a number: {}", token);
    }
}

pub fn parse_func_def_exp(lexer: &mut Lexer) -> Exp {
    let line = lexer.line();        // function
    lexer.next_token_of_kind(TOKEN_SEP_LPAREN);
    let (par_list, is_vararg) = _parse_par_list(lexer);
    lexer.next_token_of_kind(TOKEN_SEP_RPAREN);
    let block = parse_block(lexer);
    let (last_line, _) = lexer.next_token_of_kind(TOKEN_KW_END);
    FuncDefExp {
        line,
        last_line,
        par_list,
        block,
        is_vararg,
    }
}

fn _parse_par_list(lexer: &mut Lexer) -> (Vec<String>, bool) {
    match lexer.look_ahead() { 
        TOKEN_SEP_RPAREN => {
            return (vec![], false);
        },
        TOKEN_VARARG => {
            lexer.next_token();
            return (vec![], true);
        },
        _ => {}
    }
    
    let (_, name) = lexer.next_identifier();
    let mut is_vararg = false;
    let mut names = vec![name];
    while lexer.look_ahead() == TOKEN_SEP_COMMA {
        lexer.next_token();
        if lexer.look_ahead() == TOKEN_IDENTIFIER {
            let (_, name) = lexer.next_identifier();
            names.push(name);
        } else {
            lexer.next_token_of_kind(TOKEN_VARARG);
            is_vararg = true;
            break;
        }
    }
    (names, is_vararg)
}

fn parse_table_constructor_exp(lexer: &mut Lexer) -> Exp {
    let line = lexer.line();
    lexer.next_token_of_kind(TOKEN_SEP_LCURLY);
    let (key_exps, val_exps) = _parse_field_list(lexer);
    lexer.next_token_of_kind(TOKEN_SEP_RCURLY);
    let last_line = lexer.line();
    TableConstructorExp {
        line,
        last_line,
        key_exps,
        val_exps,
    }
}

fn _parse_field_list(lexer: &mut Lexer) -> (Vec<Exp>, Vec<Exp>) {
    let mut ks = vec![];
    let mut vs = vec![];
    if lexer.look_ahead() != TOKEN_SEP_RCURLY {
        let (k, v) = _parse_field(lexer);
        ks.push(k);
        vs.push(v);
        while _is_field_sep(lexer.look_ahead()) {
            lexer.next_token();
            if lexer.look_ahead() != TOKEN_SEP_RCURLY {
                let (k, v) = _parse_field(lexer);
                ks.push(k);
                vs.push(v);
            } else {
                break;
            }
        }
    }
    (ks, vs)
}

fn _is_field_sep(token_kind: Token) -> bool {
    token_kind == TOKEN_SEP_COMMA || token_kind == TOKEN_SEP_SEMI
}

fn _parse_field(lexer: &mut Lexer) -> (Exp, Exp) {
    let k: Exp;
    let v: Exp;
    if lexer.look_ahead() == TOKEN_SEP_LBRACK {
        lexer.next_token();
        k = parse_exp(lexer);
        lexer.next_token_of_kind(TOKEN_SEP_RBRACK);
        lexer.next_token_of_kind(TOKEN_OP_ASSIGN);
        v = parse_exp(lexer);
        return (k, v);
    }
    let exp = parse_exp(lexer);
    if let NameExp { ref line, ref str} = &exp {
        if lexer.look_ahead() == TOKEN_OP_ASSIGN {
            lexer.next_token();
            k = StringExp {
                line: *line,
                str: str.clone(),
            };
            v = parse_exp(lexer);
            return (k, v);
        }
    }
    (NilExp { line: 0 }, exp)
}

pub fn parse_prefix_exp(lexer: &mut Lexer) -> Exp {
    let exp: Exp = if lexer.look_ahead() == TOKEN_IDENTIFIER {
        let (line, name) = lexer.next_identifier();
        NameExp {
            line, 
            str: name,
        }
    } else {
        parse_parens_exp(lexer)
    };
    _finish_prefix_exp(lexer, exp)
}

fn _finish_prefix_exp(lexer: &mut Lexer, mut exp: Exp) -> Exp {
    loop {
        match lexer.look_ahead() {
            TOKEN_SEP_LBRACK => {
                lexer.next_token();
                let key_exp = parse_exp(lexer);
                lexer.next_token_of_kind(TOKEN_SEP_RBRACK);
                exp = TableAccessExp {
                    last_line: lexer.line(),
                    prefix_exp: Box::new(exp),
                    key_exp: Box::new(key_exp),
                };
            },
            TOKEN_SEP_DOT => {
                lexer.next_token();
                let (line, name) = lexer.next_identifier();
                let key_exp = StringExp {
                    line,
                    str: name,
                };
                exp = TableAccessExp {
                    last_line: line,
                    prefix_exp: Box::new(exp),
                    key_exp: Box::new(key_exp),
                };
            },
            TOKEN_SEP_COLON | TOKEN_SEP_LPAREN |TOKEN_SEP_LCURLY | TOKEN_STRING => {
                exp = _finish_func_call_exp(lexer, exp);            // [`:`Name] args
            },
            _ => {
                return exp;
            },
        }
    }
}

fn parse_parens_exp(lexer: &mut Lexer) -> Exp {
    lexer.next_token_of_kind(TOKEN_SEP_LPAREN);
    let exp = parse_exp(lexer);
    lexer.next_token_of_kind(TOKEN_SEP_RPAREN);
    match exp { 
        VarargExp { .. } | FuncCallExp { .. } | NameExp { .. } | TableAccessExp { .. } => {
            ParensExp {
                exp: Box::new(exp),
            }
        },
        _ => {
            exp
        }
    }
}

fn _finish_func_call_exp(lexer: &mut Lexer, prefix_exp: Exp) -> Exp {
    let name_exp = _parse_name_exp(lexer);      // [`:` Name]
    let line = lexer.line();
    let args = _parse_args(lexer);
    let last_line = lexer.line();
    FuncCallExp {
        line,
        last_line,
        prefix_exp: Box::new(prefix_exp),
        name_exp: Box::new(name_exp),
        args,
    }
}

fn _parse_name_exp(lexer: &mut Lexer) -> Exp {
    if lexer.look_ahead() == TOKEN_SEP_COLON {
        lexer.next_token();
        let (line, name) = lexer.next_identifier();
        StringExp {
            line,
            str: name,
        }
    } else {
        NilExp { line: 0 }
    }
}

fn _parse_args(lexer: &mut Lexer) -> Vec<Exp> {
    let mut args = vec![];
    match lexer.look_ahead() { 
        TOKEN_SEP_LPAREN => {
            lexer.next_token();
            if lexer.look_ahead() != TOKEN_SEP_RPAREN {
                args = parse_exp_list(lexer);
            }
            lexer.next_token_of_kind(TOKEN_SEP_RPAREN);
        },
        TOKEN_SEP_LCURLY => {
            args = vec![parse_table_constructor_exp(lexer)];
        },
        _ => {
            let (line, str) = lexer.next_token_of_kind(TOKEN_STRING);
            args = vec![
                StringExp {
                    line,
                    str,
                }
            ];
        }
    }
    args
}