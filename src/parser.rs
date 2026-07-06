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
pub enum ParserError {
    #[error("expected a value argument")]
    ExpectedValue,

    #[error("expected type {0:?}")]
    ExpectedType(ExpectedKind),

    #[error("unknown instruction {0}")]
    UnknownInstruction(String),

    #[error("unknown label {0}")]
    UnknownLabel(String),
}

pub struct Parser<'a> {
    source: &'a str,
    program: Vec<Opcode>,
    labels: HashMap<String, usize>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            program: vec![],
            labels: HashMap::new(),
        }
    }

    fn parse_t<T>(&self, num_string: &mut SplitWhitespace) -> Result<T, ParserError>
    where
        T: FromStr + RepresentableType,
    {
        num_string
            .next()
            .ok_or(ParserError::ExpectedValue)?
            .parse::<T>()
            .map_err(|_| ParserError::ExpectedType(T::kind()))
    }

    pub fn parse(mut self) -> Result<Vm, ParserError> {
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

                    "call" => {
                        let label = self.parse_t::<String>(&mut tokens)?;
                        let current_idx = self.program.len();

                        let num_args = self.parse_t::<usize>(&mut tokens)?;

                        if let Some(&address) = self.labels.get(&label) {
                            self.program.push(Opcode::Call(address, num_args));
                        } else {
                            forward_decls.entry(label).or_default().push(current_idx);
                            self.program.push(Opcode::Call(0, num_args));
                        }
                    }

                    "return" => self.program.push(Opcode::Return),

                    "halt" => self.program.push(Opcode::Halt),
                    other => return Err(ParserError::UnknownInstruction(other.to_string())),
                }
            }
        }

        if let Some((missing_label, _)) = forward_decls.into_iter().next() {
            return Err(ParserError::UnknownLabel(missing_label));
        }

        Ok(Vm::new(self.program))
    }
}
