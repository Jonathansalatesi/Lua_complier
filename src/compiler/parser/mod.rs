mod parse_block;
mod parse_exp;

use std::rc::Rc;
use super::ast::block::Block;
use super::lexer::lexer::Lexer;

use parse_block::parse_block;
use super::lexer::token::TOKEN_EOF;

pub fn parse(chunk: String, chunk_name: String) -> Rc<Block> {
    let mut lexer = Lexer::new(chunk, chunk_name);
    let block = parse_block(&mut lexer);
    lexer.next_token_of_kind(TOKEN_EOF);
    block
}