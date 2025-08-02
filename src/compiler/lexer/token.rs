pub type Token = i32;

// token kind
pub const TOKEN_EOF: i32 = 0;                               // end-of-file
pub const TOKEN_VARARG: i32 = 1;                            // ...
pub const TOKEN_SEP_SEMI: i32 = 2;                          // ;
pub const TOKEN_SEP_COMMA: i32 = 3;                         // ,
pub const TOKEN_SEP_DOT: i32 = 4;                           // .
pub const TOKEN_SEP_COLON: i32 = 5;                         // :
pub const TOKEN_SEP_LABEL: i32 = 6;                         // ::
pub const TOKEN_SEP_LPAREN: i32 = 7;                        // (
pub const TOKEN_SEP_RPAREN: i32 = 8;                        // )
pub const TOKEN_SEP_LBRACK: i32 = 9;                        // [
pub const TOKEN_SEP_RBRACK: i32 = 10;                       // ]
pub const TOKEN_SEP_LCURLY: i32 = 11;                       // {
pub const TOKEN_SEP_RCURLY: i32 = 12;                       // }
pub const TOKEN_OP_ASSIGN: i32 = 13;                        // =
pub const TOKEN_OP_MINUS: i32 = 14;                         // - (sub or unm)
pub const TOKEN_OP_WAVE: i32 = 15;                          // ~ (bnot or bxor)
pub const TOKEN_OP_ADD: i32 = 16;                           // +
pub const TOKEN_OP_MUL: i32 = 17;                           // *
pub const TOKEN_OP_DIV: i32 = 18;                           // /
pub const TOKEN_OP_IDIV: i32 = 19;                          // //
pub const TOKEN_OP_POW: i32 = 20;                           // ^
pub const TOKEN_OP_MOD: i32 = 21;                           // %
pub const TOKEN_OP_BAND: i32 = 22;                          // &
pub const TOKEN_OP_BOR: i32 = 23;                           // |
pub const TOKEN_OP_SHR: i32 = 24;                           // >>
pub const TOKEN_OP_SHL: i32 = 25;                           // <<
pub const TOKEN_OP_CONCAT: i32 = 26;                        // ..
pub const TOKEN_OP_LT: i32 = 27;                            // <
pub const TOKEN_OP_LE: i32 = 28;                            // <=
pub const TOKEN_OP_GT: i32 = 29;                            // >
pub const TOKEN_OP_GE: i32 = 30;                            // >=
pub const TOKEN_OP_EQ: i32 = 31;                            // ==
pub const TOKEN_OP_NE: i32 = 32;                            // ~=
pub const TOKEN_OP_LEN: i32 = 33;                           // #
pub const TOKEN_OP_AND: i32 = 34;                           // and
pub const TOKEN_OP_OR: i32 = 35;                            // or
pub const TOKEN_OP_NOT: i32 = 36;                           // not
pub const TOKEN_KW_BREAK: i32 = 37;                         // break
pub const TOKEN_KW_DO: i32 = 38;                            // do
pub const TOKEN_KW_ELSE: i32 = 39;                          // else
pub const TOKEN_KW_ELSEIF: i32 = 40;                        // elseif
pub const TOKEN_KW_END: i32 = 41;                           // end
pub const TOKEN_KW_FALSE: i32 = 42;                         // false
pub const TOKEN_KW_FOR: i32 = 43;                           // for
pub const TOKEN_KW_FUNCTION: i32 = 44;                      // function
pub const TOKEN_KW_GOTO: i32 = 45;                          // goto
pub const TOKEN_KW_IF: i32 = 46;                            // if
pub const TOKEN_KW_IN: i32 = 47;                            // in
pub const TOKEN_KW_LOCAL: i32 = 48;                         // local
pub const TOKEN_KW_NIL: i32 = 49;                           // nil
pub const TOKEN_KW_REPEAT: i32 = 50;                        // repeat
pub const TOKEN_KW_RETURN: i32 = 51;                        // return
pub const TOKEN_KW_THEN: i32 = 52;                          // then
pub const TOKEN_KW_TRUE: i32 = 53;                          // true
pub const TOKEN_KW_UNTIL: i32 = 54;                         // until
pub const TOKEN_KW_WHILE: i32 = 55;                         // while
pub const TOKEN_IDENTIFIER: i32 = 56;                       // identifier
pub const TOKEN_NUMBER: i32 = 57;                           // number literal
pub const TOKEN_STRING: i32 = 58;                           // string literal
pub const TOKEN_OP_UNM: i32 = TOKEN_OP_MINUS;               // unary minus
pub const TOKEN_OP_SUB: i32 = TOKEN_OP_MINUS;
pub const TOKEN_OP_BNOT: i32 = TOKEN_OP_WAVE;
pub const TOKEN_OP_BXOR: i32 = TOKEN_OP_WAVE;
pub const TOKEN_INIT_VOID: i32 = 59;                        // only for initialization

struct Keyword {
    name: &'static str,
    code: Token,
}

const fn keyword(name: &'static str, code: Token) -> Keyword {
    Keyword {
        name: name,
        code: code,
    }
}

const KEYWORDS: &'static [Keyword] = &[
    keyword("and", TOKEN_OP_AND),
    keyword("break", TOKEN_KW_BREAK),
    keyword("do", TOKEN_KW_DO),
    keyword("else", TOKEN_KW_ELSE),
    keyword("elseif", TOKEN_KW_ELSEIF),
    keyword("end", TOKEN_KW_END),
    keyword("false", TOKEN_KW_FALSE),
    keyword("for", TOKEN_KW_FOR),
    keyword("function", TOKEN_KW_FUNCTION),
    keyword("goto", TOKEN_KW_GOTO),
    keyword("if", TOKEN_KW_IF),
    keyword("in", TOKEN_KW_IN),
    keyword("local", TOKEN_KW_LOCAL),
    keyword("nil", TOKEN_KW_NIL),
    keyword("not", TOKEN_OP_NOT),
    keyword("or", TOKEN_OP_OR),
    keyword("repeat", TOKEN_KW_REPEAT),
    keyword("return", TOKEN_KW_RETURN),
    keyword("then", TOKEN_KW_THEN),
    keyword("true", TOKEN_KW_TRUE),
    keyword("until", TOKEN_KW_UNTIL),
    keyword("while", TOKEN_KW_WHILE),
];

pub fn find_keyword(name: &str) -> Option<Token> {
    for keyword in KEYWORDS {
        if keyword.name == name {
            return Some(keyword.code);
        }
    }
    None
}