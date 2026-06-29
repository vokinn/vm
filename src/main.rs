#![feature(decl_macro)]

use crate::vm::Vm;
mod errors;
mod instruction;
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

    let vm = Vm::new(source);

    let mut parsed_vm = vm.parse().unwrap();
    parsed_vm.run().unwrap();
}
