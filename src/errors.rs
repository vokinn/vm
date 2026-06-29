use crate::instruction::ExpectedKind;

#[derive(Debug)]
pub enum VmError {
    StackEmpty,
    InvalidVariable,
    DivByZero,
}

#[derive(Debug)]
pub enum ParserError {
    ExpectedValue,
    ExpectedType(ExpectedKind),
    UnknownInstruction(String),
    UnknownLabel(String),
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserError::ExpectedValue => write!(f, "expected a value argument"),
            ParserError::ExpectedType(kind) => write!(f, "expected type {:?}", kind),
            ParserError::UnknownInstruction(inst) => write!(f, "unknown instruction {}", inst),
            ParserError::UnknownLabel(label) => write!(f, "unknown label {}", label),
        }
    }
}
