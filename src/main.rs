use std::{
    collections::HashMap,
    marker::PhantomData,
    str::{FromStr, SplitWhitespace},
};

const MAX_LOCALS: usize = 64;

struct Unparsed;
struct Parsed;

#[derive(Copy, Clone, Debug, PartialEq)]
enum Opcode {
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

struct Vm<'a, T> {
    stack: Vec<i64>,
    variables: Vec<i64>,
    program: Vec<Opcode>,
    source: &'a str,
    ip: usize,

    labels: HashMap<String, usize>,
    _state: PhantomData<T>,
}

#[derive(Debug)]
enum VmError {
    StackEmpty,
    InvalidVariable,
    DivByZero,
}

#[derive(Debug)]
enum ExpectedKind {
    I64,
    Usize,
    String,
}

trait RepresentableType {
    fn kind() -> ExpectedKind;
}

impl RepresentableType for i64 {
    fn kind() -> ExpectedKind {
        ExpectedKind::I64
    }
}

impl RepresentableType for usize {
    fn kind() -> ExpectedKind {
        ExpectedKind::Usize
    }
}

impl RepresentableType for String {
    fn kind() -> ExpectedKind {
        ExpectedKind::String
    }
}

#[derive(Debug)]
enum ParserError {
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

impl<'a> Vm<'a, Unparsed> {
    fn new(source: &'a str) -> Self {
        Self {
            stack: vec![],
            variables: vec![0; MAX_LOCALS],
            program: vec![],
            source,
            ip: 0,

            labels: HashMap::new(),
            _state: PhantomData,
        }
    }

    fn parse_t<T: FromStr + RepresentableType>(
        &self,
        num_string: &mut SplitWhitespace,
    ) -> Result<T, ParserError> {
        num_string
            .next()
            .ok_or(ParserError::ExpectedValue)?
            .parse::<T>()
            .map_err(|_| ParserError::ExpectedType(T::kind()))
    }

    fn parse(mut self) -> Result<Vm<'a, Parsed>, ParserError> {
        let mut forward_decls: HashMap<String, Vec<usize>> = HashMap::new();

        for line in self.source.lines() {
            let mut tokens = line.split_whitespace();

            if let Some(mut first_token) = tokens.next() {
                if first_token.ends_with(':') {
                    let label_name = first_token[..first_token.len() - 1].to_string();
                    let label_address = self.program.len();
                    self.labels.insert(label_name.clone(), label_address);

                    if let Some(pending_indices) = forward_decls.remove(&label_name) {
                        for index in pending_indices {
                            match &mut self.program[index] {
                                Opcode::Jump(addr) | Opcode::JumpIfZero(addr) => {
                                    *addr = label_address
                                }
                                _ => {}
                            }
                        }
                    }

                    if let Some(next_token) = tokens.next() {
                        first_token = next_token;
                    } else {
                        continue;
                    }
                }

                match first_token {
                    "push" => self
                        .program
                        .push(Opcode::Push(self.parse_t::<i64>(&mut tokens)?)),

                    "add" => self.program.push(Opcode::Add),
                    "subtract" => self.program.push(Opcode::Subtract),
                    "multiply" => self.program.push(Opcode::Multiply),
                    "divide" => self.program.push(Opcode::Divide),

                    "print" => self.program.push(Opcode::Print),
                    "debug" => self.program.push(Opcode::Debug),
                    "duplicate" => self.program.push(Opcode::Duplicate),

                    "load" => self
                        .program
                        .push(Opcode::Load(self.parse_t::<usize>(&mut tokens)?)),

                    "store" => self
                        .program
                        .push(Opcode::Store(self.parse_t::<usize>(&mut tokens)?)),

                    "jump" => {
                        let label = self.parse_t::<String>(&mut tokens)?;
                        let current_idx = self.program.len();

                        if let Some(&address) = self.labels.get(&label) {
                            self.program.push(Opcode::Jump(address));
                        } else {
                            forward_decls.entry(label).or_default().push(current_idx);
                            self.program.push(Opcode::Jump(0));
                        }
                    }

                    "jumpifzero" => {
                        let label = self.parse_t::<String>(&mut tokens)?;
                        let current_idx = self.program.len();

                        if let Some(&address) = self.labels.get(&label) {
                            self.program.push(Opcode::JumpIfZero(address));
                        } else {
                            forward_decls.entry(label).or_default().push(current_idx);
                            self.program.push(Opcode::JumpIfZero(0));
                        }
                    }

                    "halt" => self.program.push(Opcode::Halt),
                    other => return Err(ParserError::UnknownInstruction(other.to_string())),
                }
            }
        }

        if let Some((missing_label, _)) = forward_decls.into_iter().next() {
            return Err(ParserError::UnknownLabel(missing_label));
        }

        Ok(Vm {
            stack: self.stack,
            variables: self.variables,
            program: self.program,
            source: self.source,
            ip: self.ip,

            labels: self.labels,
            _state: PhantomData::<Parsed>,
        })
    }
}

impl<'a> Vm<'a, Parsed> {
    fn call(&mut self, opcode: Opcode) -> Result<(), VmError> {
        match opcode {
            Opcode::Push(n) => self.stack.push(n),
            Opcode::Add => {
                let a = self.stack.pop().ok_or(VmError::StackEmpty)?;
                let b = self.stack.pop().ok_or(VmError::StackEmpty)?;

                self.stack.push(a + b);
            }

            Opcode::Subtract => {
                let a = self.stack.pop().ok_or(VmError::StackEmpty)?;
                let b = self.stack.pop().ok_or(VmError::StackEmpty)?;

                self.stack.push(b - a);
            }

            Opcode::Multiply => {
                let a = self.stack.pop().ok_or(VmError::StackEmpty)?;
                let b = self.stack.pop().ok_or(VmError::StackEmpty)?;

                self.stack.push(a * b);
            }

            Opcode::Divide => {
                let a = self.stack.pop().ok_or(VmError::StackEmpty)?;
                let b = self.stack.pop().ok_or(VmError::StackEmpty)?;

                if a == 0 {
                    return Err(VmError::DivByZero);
                }

                self.stack.push(b / a);
            }

            Opcode::Print => {
                let top = self.stack.pop().ok_or(VmError::StackEmpty)?;
                println!("{}", top);
            }

            Opcode::Debug => {
                println!("{:?}", self.stack);
            }

            Opcode::Duplicate => {
                let top = self.stack.last().ok_or(VmError::StackEmpty)?;
                self.stack.push(*top);
            }

            Opcode::Store(n) => {
                let top = self.stack.pop().ok_or(VmError::StackEmpty)?;
                self.variables[n] = top;
            }

            Opcode::Load(n) => {
                let top = self.variables.get(n).ok_or(VmError::InvalidVariable)?;
                self.stack.push(*top);
            }

            _ => (),
        }

        Ok(())
    }

    fn run(&mut self) -> Result<(), VmError> {
        while self.ip < self.program.len() {
            let opcode = self.program[self.ip];
            self.ip += 1;

            match opcode {
                Opcode::Jump(n) => self.ip = n,
                Opcode::JumpIfZero(n) => {
                    if self.stack.pop().ok_or(VmError::StackEmpty)? == 0 {
                        self.ip = n;
                    };
                }

                Opcode::Halt => break,
                _ => self.call(opcode)?,
            }
        }

        Ok(())
    }
}

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
    println!("{:?}", parsed_vm.program);

    parsed_vm.run().unwrap();
}
