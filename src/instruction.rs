#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Opcode {
    Push(i64),

    Add,
    Subtract,
    Multiply,
    Divide,

    Print,
    Debug,
    Halt,

    Jump(usize),
    JumpIfZero(usize),
    Duplicate,

    Load(usize),
    Store(usize),
}

#[derive(Debug)]
pub enum ExpectedKind {
    I64,
    Usize,
    String,
}

pub trait RepresentableType {
    fn kind() -> ExpectedKind;
}

pub macro impl_representable($($type:ty => $kind:ident),* $(,)?) {
    $(
        impl RepresentableType for $type {
            fn kind() -> ExpectedKind {
                ExpectedKind::$kind
            }
        }
    )*
}

impl_representable!(
    i64 => I64,
    usize => Usize,
    String => String,
);
