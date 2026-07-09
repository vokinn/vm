use crate::{
    instruction::{ExpectedKind, Opcode, RepresentableType},
    vm::Vm,
};

use std::{
    collections::HashMap,
    str::{FromStr, SplitWhitespace},
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError<'a> {
    #[error("expected a value argument")]
    ExpectedValue,

    #[error("expected type {0:?}")]
    ExpectedType(ExpectedKind),

    #[error("unknown instruction {0}")]
    UnknownInstruction(&'a str),

    #[error("unknown label {0}")]
    UnknownLabel(&'a str),
}

pub struct Parser<'a> {
    source: &'a str,
    program: Vec<Opcode>,
    labels: HashMap<&'a str, usize>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            program: vec![],
            labels: HashMap::new(),
        }
    }

    fn next_arg<T>(&self, num_string: &mut SplitWhitespace) -> Result<T, ParseError<'a>>
    where
        T: FromStr + RepresentableType,
    {
        num_string
            .next()
            .ok_or(ParseError::ExpectedValue)?
            .parse::<T>()
            .map_err(|_| ParseError::ExpectedType(T::kind()))
    }

    pub fn parse(mut self) -> Result<Vm, ParseError<'a>> {
        let mut forward_decls: HashMap<&'a str, Vec<usize>> = HashMap::new();

        for line in self.source.lines() {
            let mut tokens = line.split_whitespace();

            if let Some(mut first_token) = tokens.next() {
                if first_token.ends_with(':') {
                    let label_name = &first_token[..first_token.len() - 1];
                    let label_address = self.program.len();
                    self.labels.insert(label_name, label_address);

                    if let Some(pending_indices) = forward_decls.remove(label_name) {
                        for index in pending_indices {
                            match &mut self.program[index] {
                                Opcode::Jump(addr)
                                | Opcode::JumpIfZero(addr)
                                | Opcode::Call(addr, _) => *addr = label_address,

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
                        .push(Opcode::Push(self.next_arg::<i64>(&mut tokens)?)),

                    "add" => self.program.push(Opcode::Add),
                    "subtract" => self.program.push(Opcode::Subtract),
                    "multiply" => self.program.push(Opcode::Multiply),
                    "divide" => self.program.push(Opcode::Divide),

                    "print" => self.program.push(Opcode::Print),
                    "debug" => self.program.push(Opcode::Debug),
                    "duplicate" => self.program.push(Opcode::Duplicate),

                    "load" => self
                        .program
                        .push(Opcode::Load(self.next_arg::<usize>(&mut tokens)?)),

                    "store" => self
                        .program
                        .push(Opcode::Store(self.next_arg::<usize>(&mut tokens)?)),

                    "jump" => {
                        let label = tokens.next().ok_or(ParseError::ExpectedValue)?;
                        let current_idx = self.program.len();

                        if let Some(&address) = self.labels.get(label) {
                            self.program.push(Opcode::Jump(address));
                        } else {
                            forward_decls.entry(label).or_default().push(current_idx);
                            self.program.push(Opcode::Jump(0));
                        }
                    }

                    "jumpifzero" => {
                        let label = tokens.next().ok_or(ParseError::ExpectedValue)?;
                        let current_idx = self.program.len();

                        if let Some(&address) = self.labels.get(label) {
                            self.program.push(Opcode::JumpIfZero(address));
                        } else {
                            forward_decls.entry(label).or_default().push(current_idx);
                            self.program.push(Opcode::JumpIfZero(0));
                        }
                    }

                    "call" => {
                        let label = tokens.next().ok_or(ParseError::ExpectedValue)?;
                        let current_idx = self.program.len();

                        let num_args = self.next_arg::<usize>(&mut tokens)?;

                        if let Some(&address) = self.labels.get(label) {
                            self.program.push(Opcode::Call(address, num_args));
                        } else {
                            forward_decls.entry(label).or_default().push(current_idx);
                            self.program.push(Opcode::Call(0, num_args));
                        }
                    }

                    "return" => self.program.push(Opcode::Return),

                    "halt" => self.program.push(Opcode::Halt),
                    other => return Err(ParseError::UnknownInstruction(other)),
                }
            }
        }

        if let Some((missing_label, _)) = forward_decls.into_iter().next() {
            return Err(ParseError::UnknownLabel(missing_label));
        }

        Ok(Vm::new(self.program))
    }
}
