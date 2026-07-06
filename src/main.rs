#![feature(decl_macro)]
use crate::parser::Parser;

mod errors;
mod instruction;
mod parser;
mod vm;

fn main() {
    let source = "
        jump main

        factorial:
            load 0
            jumpifzero base
            load 0
            load 0
            push 1
            subtract
            call factorial 1
            multiply
            return

        base:
            push 1
            return

        main:
            push 5
            call factorial 1
            print
            halt
    ";

    let parser = Parser::new(source);
    let mut vm = parser.parse().unwrap();

    vm.run().unwrap();
}
