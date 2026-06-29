#![feature(decl_macro)]
use crate::parser::Parser;

mod errors;
mod instruction;
mod parser;
mod vm;

fn main() {
    let source = "
        push 5
        store 0
        two: load 0
        duplicate
        jumpifzero end
        print
        load 0
        push 1
        subtract
        store 0
        jump two
        end: halt
    ";

    let parser = Parser::new(source);
    let mut vm = parser.parse().unwrap();

    vm.run().unwrap();
}
