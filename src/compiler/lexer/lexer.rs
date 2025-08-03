use regex::Regex;
use super::token::*;

#[allow(dead_code)]
pub struct Lexer {
    chunk: String,
    chunk_name: String,
    line: i32,
    next_token_: String,
    next_token_kind: Token,
    next_token_line: i32,
}

impl Lexer {
    pub fn new(chunk: String, chunk_name: String) -> Self {
        Lexer {
            chunk: chunk,
            chunk_name: chunk_name,
            line: 1,
            next_token_: "".to_owned(),
            next_token_kind: TOKEN_INIT_VOID,
            next_token_line: -1,
        }
    }

    fn get_nth_char(&self, inx: usize) -> char {
        self.chunk.chars().nth(inx).unwrap()
    }
    
    pub fn next_token(&mut self) -> (i32, Token, String) {
        if self.next_token_line > 0 {
            let line = self.next_token_line;
            let kind = self.next_token_kind;
            let token = self.next_token_.clone();
            self.line = self.next_token_line;
            self.next_token_line = 0;
            return (line, kind, token);
        }
        
        self.skip_whitespaces();
        if self.chunk.len() == 0 {
            return (self.line, TOKEN_EOF, "EOF".to_string());
        }
        match self.get_nth_char(0) {
            ';' => {
                self.next(1);
                return (self.line, TOKEN_SEP_SEMI, ";".to_string());
            },
            ',' => {
                self.next(1);
                return (self.line, TOKEN_SEP_COMMA, ",".to_string());
            },
            '(' => {
                self.next(1);
                return (self.line, TOKEN_SEP_LPAREN, "(".to_string());
            },
            ')' => {
                self.next(1);
                return (self.line, TOKEN_SEP_RPAREN, ")".to_string());
            },
            ']' => {
                self.next(1);
                return (self.line, TOKEN_SEP_RBRACK, "]".to_string());
            },
            '{' => {
                self.next(1);
                return (self.line, TOKEN_SEP_LCURLY, "{".to_string());
            },
            '}' => {
                self.next(1);
                return (self.line, TOKEN_SEP_RCURLY, "}".to_string());
            },
            '+' => {
                self.next(1);
                return (self.line, TOKEN_OP_ADD, "+".to_string());
            },
            '-' => {
                self.next(1);
                return (self.line, TOKEN_OP_MINUS, "-".to_string());
            },
            '*' => {
                self.next(1);
                return (self.line, TOKEN_OP_MUL, "*".to_string());
            },
            '^' => {
                self.next(1);
                return (self.line, TOKEN_OP_POW, "^".to_string());
            },
            '%' => {
                self.next(1);
                return (self.line, TOKEN_OP_MOD, "%".to_string());
            },
            '&' => {
                self.next(1);
                return (self.line, TOKEN_OP_BAND, "&".to_string());
            },
            '|' => {
                self.next(1);
                return (self.line, TOKEN_OP_BOR, "|".to_string());
            },
            '#' => {
                self.next(1);
                return (self.line, TOKEN_OP_LEN, "#".to_string());
            },
            ':' => {
                if self.test("::") {
                    self.next(2);
                    return (self.line, TOKEN_SEP_LABEL, "::".to_string());
                }
                self.next(1);
                return (self.line, TOKEN_SEP_COLON, ":".to_string());
            },
            '/' => {
                if self.test("//") {
                    self.next(2);
                    return (self.line, TOKEN_OP_IDIV, "//".to_string());
                }
                self.next(1);
                return (self.line, TOKEN_OP_DIV, "/".to_string());
            },
            '~' => {
                if self.test("~=") {
                    self.next(2);
                    return (self.line, TOKEN_OP_NE, "~=".to_string());
                }
                self.next(1);
                return (self.line, TOKEN_OP_WAVE, "~".to_string());
            },
            '=' => {
                if self.test("==") {
                    self.next(2);
                    return (self.line, TOKEN_OP_EQ, "==".to_string());
                }
                self.next(1);
                return (self.line, TOKEN_OP_ASSIGN, "=".to_string());
            },
            '<' => {
                return if self.test("<<") {
                    self.next(2);
                    (self.line, TOKEN_OP_SHL, "<<".to_string())
                } else if self.test("<=") {
                    self.next(2);
                    (self.line, TOKEN_OP_LE, "<=".to_string())
                } else {
                    self.next(1);
                    (self.line, TOKEN_OP_LT, "<".to_string())
                }
            },
            '>' => {
                return if self.test(">>") {
                    self.next(2);
                    (self.line, TOKEN_OP_SHR, ">>".to_string())
                } else if self.test(">=") {
                    self.next(2);
                    (self.line, TOKEN_OP_GE, ">=".to_string())
                } else {
                    self.next(1);
                    (self.line, TOKEN_OP_GT, ">".to_string())
                }
            },
            '.' => {
                if self.test("...") {
                    self.next(3);
                    return (self.line, TOKEN_VARARG, "...".to_string());
                } else if self.test("..") {
                    self.next(2);
                    return (self.line, TOKEN_OP_CONCAT, "..".to_string());
                } else if self.chunk.len() == 1 || !is_digit(self.get_nth_char(1)) { 
                    self.next(1);
                    return (self.line, TOKEN_SEP_DOT, ".".to_string());
                }
            },
            '[' => {
                return if self.test("[[") || self.test("[=") {
                    (self.line, TOKEN_STRING, self.scan_long_string())
                } else {
                    self.next(1);
                    (self.line, TOKEN_SEP_LBRACK, "[".to_string())
                };
            },
            '\'' | '\"' => {
                return (self.line, TOKEN_STRING, self.scan_short_string());
            },
            _ => {},
        }
        
        let c = self.get_nth_char(0);
        if c == '.' || is_digit(c) {
            let token = self.scan_number();
            return (self.line, TOKEN_NUMBER, token);
        }
        if c == '_' || is_letter(c) {
            let token = self.scan_identifier();
            if let Some(kind) = find_keyword(&token) {
                return (self.line, kind, token);
            } else {
                return (self.line, TOKEN_IDENTIFIER, token);
            }
        }
        
        panic!("Unexpected symbol near {}", c);
    }
    
    pub fn look_ahead(&mut self) -> Token {
        if self.next_token_line > 0 {
            return self.next_token_kind;
        }
        let current_line = self.line;
        let (line, kind, token) = self.next_token();
        self.line = current_line;
        self.next_token_line = line;
        self.next_token_kind = kind;
        self.next_token_ = token;
        kind
    }
    
    pub fn next_token_of_kind(&mut self, kind: Token) -> (i32, String) {
        let (line, _kind, token) = self.next_token();
        if _kind != kind {
            panic!("Syntax error near <{}> in line {}", token, line);
        }
        (line, token)
    }
    
    pub fn next_identifier(&mut self) -> (i32, String) {
        self.next_token_of_kind(TOKEN_IDENTIFIER)
    }
    
    pub fn line(&self) -> i32 {
        self.line
    }
    
    fn skip_whitespaces(&mut self) {
        while self.chunk.len() > 0 {
            if self.test("--") {
                self.skip_comment();
            } else if self.test("\r\n") || self.test("\n\r") {
                self.next(2);
                self.line += 1;
            } else if is_new_line(self.get_nth_char(0)) {
                self.next(1);
                self.line += 1;
            } else if is_white_space(self.get_nth_char(0)) {
                self.next(1);
            } else {
                break;
            }
        }
    }
    
    fn test(&self, s: &str) -> bool {
        self.chunk.starts_with(s)
    }
    
    fn next(&mut self, n: usize) {
        let val = self.chunk.as_str();
        self.chunk = String::from(&val[n..]);
    }
    
    fn skip_comment(&mut self) {
        self.next(2);           // skip --
        if self.test("[") {     // long comment ?
            let re = Regex::new(r"^\[=*\[").unwrap();
            let result = re.captures(self.chunk.as_str());
            if let Some(_) = result{
                self.scan_long_string();
                return;
            }
        }

        // short comment
        while self.chunk.len() > 0 && !is_new_line(self.get_nth_char(0)) {
            self.next(1);
        }
    }

    fn scan_long_string(&mut self) -> String {
        let re_opening_long_bracket =  Regex::new(r"^\[=*\[").unwrap();
        if let Some(cat) = re_opening_long_bracket.find(&self.chunk) {
            let len_end_flag = cat.end() - cat.start();
            
            // create the end flag of long string.
            let mut str_end_flag = "".to_owned();
            str_end_flag.push(']');
            for _ in 2..len_end_flag {
                str_end_flag.push('=');
            }
            str_end_flag.push(']');
            
            if let Some(pos) = self.chunk.find(&str_end_flag) {
                let str_tmp = String::from(&self.chunk[cat.end()..pos]);
                self.next(pos + len_end_flag);
                let re_new_line = Regex::new(r"\r\n|\n\r|\n|\r").unwrap();
                let count_backslash = re_new_line.find_iter(&str_tmp).count();
                self.line += count_backslash as i32;
                if str_tmp.len() > 0 && str_tmp.chars().nth(0).unwrap() == '\n' {
                    return (&str_tmp[1..]).to_owned();
                }
                str_tmp
            } else {
                panic!("Unfinished long string or comment!\n");
            }
        } else {
            panic!("Invalid long string delimiter near {}!\n", (self.chunk.as_str())[0..2].to_owned());
        }
    }
    
    fn scan_short_string(&mut self) -> String {
        let re_short_str = Regex::new(r#"(?s)(^"(\\\\|\\"|\\\n|\\z\s*|[^"\n])*")|(^'(\\\\|\\'|\\\n|\\z\s*|[^'\n])*')"#).unwrap();
        if let Some(cap) = re_short_str.find(&self.chunk) {
            let step = cap.end() - cap.start();
            let mut str_ = String::from(&self.chunk[1..(step - 1)]);
            self.next(step);
            if str_.contains("\\") {
                let re_new_line =  Regex::new(r"\r\n|\n\r|\n|\r").unwrap();
                let count = re_new_line.find_iter(&str_).count();
                self.line += count as i32;
                str_ = self.escape(&str_);
            }
            return str_;
        }
        
        panic!("Unfinished string");
    }
    
    fn escape(&self, mut s: &str) -> String {
        let mut buf: Vec<char> = vec![];
        while s.len() > 0 {
            if s.chars().nth(0).unwrap() != '\\' {
                buf.push(s.chars().nth(0).unwrap());
                s = &s[1..];
                continue;
            }
            if s.len() == 1 {
                panic!("Unfinished string");
            }
            match s.chars().nth(1).unwrap() { 
                'a' => {
                    // belling char ASCII code: 7
                    buf.push(7 as char);
                    s = &s[2..];
                },
                'b' => {
                    buf.push(8 as char);
                    s = &s[2..];
                },
                'f' => {
                    buf.push(12 as char);
                    s = &s[2..];
                },
                'n' => {
                    buf.push('\n');
                    s = &s[2..];
                },
                '\n' => {
                    buf.push('\n');
                    s = &s[2..];
                },
                'r' => {
                    buf.push('\r');
                    s = &s[2..];
                },
                't' => {
                    buf.push(9 as char);
                    s = &s[2..];
                },
                'v' => {
                    buf.push(11 as char);
                    s = &s[2..];
                },
                '\"' => {
                    buf.push('\"');
                    s = &s[2..];
                },
                '\'' => {
                    buf.push('\'');
                    s = &s[2..];
                },
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                    // \ddd
                    let re_dec_escape_seq = Regex::new(r"^\\[0-9]{1,3}").unwrap();
                    if let Some(cap) = re_dec_escape_seq.find(s) {
                        let found = cap.as_str();
                        if let Ok(d) = (&found[1..]).parse::<u32>() {
                            if d <= 0xFF {
                                buf.push(d as u8 as char);
                                s = &s[found.len()..];
                                continue;
                            }
                        }
                        panic!("Decimal escape too large near \'{}\'", found);
                    }
                },
                'x' => {
                    // \xXX
                    let re_hex_escape_seq = Regex::new(r"^\\x[0-9a-fA-F]{2}").unwrap();
                    if let Some(cap) = re_hex_escape_seq.find(s) {
                        let found = cap.as_str();
                        if let Ok(d) = (&found[2..]).parse::<u32>() {
                            buf.push(d as u8 as char);
                            s = &s[found.len()..];
                            continue;
                        }
                    }
                },
                'u' => {
                    // \u{XXX}
                    let re_unicode_escape_seq = Regex::new(r"^\\u\{[0-9a-fA-F]+\}").unwrap();
                    if let Some(cap) = re_unicode_escape_seq.find(s) {
                        let found = cap.as_str();
                        if let Ok(d) = (&found[3..found.len() - 1]).parse::<u32>() {
                            if d <= 0x10FFFF {
                                buf.push(((d >> 24) & 0xFF) as u8 as char);
                                buf.push(((d >> 16) & 0xFF) as u8 as char);
                                buf.push(((d >> 8) & 0xFF) as u8 as char);
                                buf.push((d & 0xFF) as u8 as char);
                                s = &s[found.len()..];
                                continue;
                            }
                        }
                        panic!("UTF-8 value too large near \"{}\"", found);
                    }
                },
                'z' => {
                    s = &s[2..];
                    while s.len() > 0 && is_white_space(s.chars().nth(0).unwrap()) {
                        s = &s[1..];
                    }
                    continue;
                },
                _ =>panic!("Invalid escape sequence near \'\\{}\'", s.chars().nth(1).unwrap()),
            }
        }
        buf.iter().collect()
    }
    
    fn scan_number(&mut self) -> String {
        let re_number = Regex::new(r"^0[xX][0-9a-fA-F]*(\.[0-9a-fA-F]*)?([pP][+|-]?[0-9]+)?|^[0-9]*(\.[0-9]*)?([eE][+|-]?[0-9]+)?").unwrap();
        self.scan(&re_number)
    }
    
    fn scan_identifier(&mut self) -> String {
        let re_identifier = Regex::new(r"^[_\d\w]+").unwrap();
        self.scan(&re_identifier)
    }
    
    fn scan(&mut self, re: &Regex) -> String {
        if let Some(cap) = re.find(&self.chunk) {
            let token = cap.as_str().to_owned();
            self.next(token.len());
            return token;
        }
        panic!("Unreachable!");
    }
}

fn is_white_space(c: char) -> bool {
    match c { 
        '\t' => true,
        '\n' => true,
        '\r' => true,
        ' ' => true,
        _ => false,
    }
}

fn is_new_line(c: char) -> bool {
    c == '\n' || c == '\r'
}

fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

fn is_letter(c: char) -> bool {
    c >= 'A' && c <= 'Z' ||  c >= 'a' && c <= 'z'
}

#[cfg(test)]
mod tests{
    use regex::Regex;

    #[test]
    fn test_regex0_0() {
        let re = Regex::new(r"\[=*\[").unwrap();
        let example = "rt".to_owned();

        // only capture the first substring that has been matched.
        if let Some(caps) = re.captures(&example) {
            assert_eq!(1, caps.len());
            assert_eq!("[=======[", &caps[0]);
            assert_eq!(9, (&caps[0]).len());
        } else {
            println!("warning.");
        }
    }

    #[test]
    fn test_regex0() {
        let re = Regex::new(r"\[=*\[").unwrap();
        let example = "rt [=======[\
        \
       [======[
        ".to_owned();

        // only capture the first substring that has been matched.
        if let Some(caps) = re.captures(&example) {
            assert_eq!(1, caps.len());
            assert_eq!("[=======[", &caps[0]);
            assert_eq!(9, (&caps[0]).len());
        }

        // return all the substrings
        let vals: Vec<&str> = re.find_iter(&example).map(|x| x.as_str()).collect();
        assert_eq!(vals, vec![
            "[=======[",
            "[======[",
        ]);
    }

    #[test]
    fn test_regex1() {
        let re = Regex::new(r"Homer (.)\. (Simpson)").unwrap();
        let hay = "Homer J. Simpson";
        let Some(caps) = re.captures(hay) else {
            return;
        };
        assert_eq!("Homer J. Simpson", &caps[0]);
        assert_eq!("J", &caps[1]);
        assert_eq!("Simpson", &caps[2]);
    }

    #[test]
    fn test_regex2() {
        let re = Regex::new(r"Homer (?<center>.)\. (Simpson)").unwrap();
        let hay = "Homer J. Simpson";
        let Some(caps) = re.captures(hay) else {
            return;
        };
        assert_eq!("Homer J. Simpson", &caps[0]);
        assert_eq!("J", &caps[1]);
        assert_eq!("J", &caps["center"]);
        assert_eq!("Simpson", &caps[2]);
    }

    #[test]
    fn test_string_index() {
        let sample = "you are victorious".to_owned();
        let c0 = sample.chars().nth(0).unwrap();
        let c1 = sample.chars().nth(1).unwrap();
        assert_eq!(c0, 'y');
        assert_eq!(c1, 'o');
    }

    #[test]
    fn test_string_next() {
        let sample = "you are victorious".to_owned();
        let ref0 = sample.as_str();
        let sub_str = (&ref0[1..]).to_owned();
        assert_eq!(sub_str, "ou are victorious".to_owned());
    }

    #[test]
    fn test_regex_find() {
        let re = Regex::new(r"\[=*\[").unwrap();
        let example = "rt [=======[\
        \
       [======[
        ".to_owned();
        
        if let Some(cat) = re.find(&example) {
            assert_eq!(cat.start(), 3);
            assert_eq!(cat.end(), 12);
        }
    }

    #[test]
    fn test_string_find() {
        let sub_string = "love".to_owned();
        let example = "I love u forever love.".to_owned();

        if let Some(cat) = example.find(&sub_string) {
            assert_eq!(cat, 2);
        }
    }

    #[test]
    fn test_newline_regex_find() {
        let re = Regex::new(r"\r\n|\n\r|\n|\r").unwrap();
        let example = "y\n\rx\r\nu\nk\rj";
        let count = re.find_iter(&example).count();
        assert_eq!(count, 4);
    }
    
    #[test]
    fn test_decimal_number() {
        // decimal float format:
        // -0.23132      [+|-]?[0-9]+(\.[0-9]*)?
        // -0.23132      [+|-]?[0-9]+(\.[0-9]*)?([e|E][+|-][0-9]+)?
        // .8989e-1      [+|-]?([0-9]+(\.[0-9]*)?|\.[0-9]+)([e|E][+|-][0-9]+)?
        let re = Regex::new(r"[+|-]?([0-9]+(\.[0-9]*)?|\.[0-9]+)([e|E][+|-][0-9]+)?").unwrap();
        let example = "0.234 -2. .56 89e-3 3E+5";
        for cap in re.find_iter(example) {
            println!("{}", &example[cap.start()..cap.end()]);
        }
    }
    
    #[test]
    fn test_short_string() {
        let re = Regex::new(r#"(?m)(^"(\\\\|\\"|\\\n|\\z\s*|[^"\n])*")|(^'(\\\\|\\'|\\\n|\\z\s*|[^'\n])*')"#).unwrap();
        let example = "\" you are my true love\",\n\'I love u forever.\'";
        let cap = re.find(example).unwrap();
        // println!("{}", &example[cap.start()..cap.end()]);
        for cap in re.find_iter(example) {
            println!("{}", cap.as_str());
        }
    }
}