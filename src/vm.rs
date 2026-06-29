use std::{
    collections::HashMap,
    marker::PhantomData,
    str::{FromStr, SplitWhitespace},
};

use crate::{
    errors::{ParserError, VmError},
    instruction::{Opcode, RepresentableType},
};

pub const MAX_LOCALS: usize = 64;

pub struct Unparsed;
pub struct Parsed;

pub struct Vm<'a, T> {
    stack: Vec<i64>,
    variables: Vec<i64>,
    program: Vec<Opcode>,
    source: &'a str,
    ip: usize,

    labels: HashMap<String, usize>,
    _state: PhantomData<T>,
}

impl<'a> Vm<'a, Unparsed> {
    pub fn new(source: &'a str) -> Self {
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

    pub fn parse(mut self) -> Result<Vm<'a, Parsed>, ParserError> {
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

    pub fn run(&mut self) -> Result<(), VmError> {
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
