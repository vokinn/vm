use std::marker::PhantomData;

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

    _state: PhantomData<T>,
}

#[derive(Debug)]
enum VmError {
    StackEmpty,
    InvalidVariable,
    DivByZero,
}

#[derive(Debug)]
enum ParserError {
    ExpectedValue,
    ExpectedI64,
    ExpectedUsize,
}

impl<'a> Vm<'a, Unparsed> {
    fn new(source: &'a str) -> Self {
        Self {
            stack: vec![],
            variables: vec![0; MAX_LOCALS],
            program: vec![],
            source,
            ip: 0,
            _state: PhantomData,
        }
    }
    //
    // fn parse_i64(num_string: &str) -> Result<i64, ParserError> {
    //     num_string
    //         .parse::<i64>()
    //         .map_err(|_| ParserError::ExpectedI64)
    // }

    fn parse(mut self) -> Result<Vm<'a, Parsed>, ParserError> {
        for line in self.source.lines() {
            let mut tokens = line.split_whitespace();

            if let Some(opcode) = tokens.next() {
                match opcode {
                    "push" => {
                        let value = tokens
                            .next()
                            .ok_or(ParserError::ExpectedValue)?
                            .parse::<i64>()
                            .map_err(|_| ParserError::ExpectedI64)?;

                        self.program.push(Opcode::Push(value));
                    }

                    "add" => self.program.push(Opcode::Add),
                    "subtract" => self.program.push(Opcode::Subtract),
                    "multiply" => self.program.push(Opcode::Multiply),
                    "divide" => self.program.push(Opcode::Divide),

                    "print" => self.program.push(Opcode::Print),
                    "debug" => self.program.push(Opcode::Debug),
                    "duplicate" => self.program.push(Opcode::Duplicate),

                    "load" => {
                        let value = tokens
                            .next()
                            .ok_or(ParserError::ExpectedValue)?
                            .parse::<usize>()
                            .map_err(|_| ParserError::ExpectedUsize)?;

                        self.program.push(Opcode::Load(value));
                    }

                    "store" => {
                        let value = tokens
                            .next()
                            .ok_or(ParserError::ExpectedValue)?
                            .parse::<usize>()
                            .map_err(|_| ParserError::ExpectedUsize)?;

                        self.program.push(Opcode::Store(value));
                    }

                    "jump" => {
                        let value = tokens
                            .next()
                            .ok_or(ParserError::ExpectedValue)?
                            .parse::<usize>()
                            .map_err(|_| ParserError::ExpectedUsize)?;

                        self.program.push(Opcode::Jump(value));
                    }

                    "jumpifzero" => {
                        let value = tokens
                            .next()
                            .ok_or(ParserError::ExpectedValue)?
                            .parse::<usize>()
                            .map_err(|_| ParserError::ExpectedUsize)?;

                        self.program.push(Opcode::JumpIfZero(value));
                    }

                    "halt" => self.program.push(Opcode::Halt),

                    _ => (),
                }
            }
        }

        Ok(Vm {
            stack: self.stack,
            variables: self.variables,
            program: self.program,
            source: self.source,
            ip: self.ip,

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
        push 6
        push 7
        add
        print
        push 8
        subtract
        print
        halt
        ";

    let vm = Vm::new(source);

    let mut parsed_vm = vm.parse().unwrap();
    println!("{:?}", parsed_vm.program);

    parsed_vm.run().unwrap();
}
