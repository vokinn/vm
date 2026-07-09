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

#[derive(Default)]
struct LabelInfo {
    address: usize,
    max_frame_size: usize,
}

pub struct Parser<'a> {
    source: &'a str,
    program: Vec<Opcode>,
    labels: HashMap<&'a str, LabelInfo>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            program: vec![],
            labels: HashMap::new(),
        }
    }

    fn next_arg<T>(&self, tokens: &mut SplitWhitespace) -> Result<T, ParseError<'a>>
    where
        T: FromStr + RepresentableType,
    {
        tokens
            .next()
            .ok_or(ParseError::ExpectedValue)?
            .parse::<T>()
            .map_err(|_| ParseError::ExpectedType(T::kind()))
    }

    fn patch_forward_calls(&mut self, label: &str, pending_indices: &[usize], frame_size: usize) {
        if let Some(info) = self.labels.get(label) {
            let final_frame_size = frame_size.max(info.max_frame_size);

            for &index in pending_indices {
                match &mut self.program[index] {
                    Opcode::Jump(addr) | Opcode::JumpIfZero(addr) => {
                        *addr = info.address;
                    }

                    Opcode::Call(addr, _, f_size) => {
                        *addr = info.address;
                        *f_size = final_frame_size;
                    }

                    _ => (),
                }
            }
        }
    }

    pub fn parse(mut self) -> Result<Vm, ParseError<'a>> {
        let mut forward_decls: HashMap<&'a str, Vec<usize>> = HashMap::new();
        let mut current_fn: Option<&'a str> = None;

        for line in self.source.lines() {
            let mut tokens = line.split_whitespace();

            if let Some(mut first_token) = tokens.next() {
                if first_token.ends_with(':') {
                    let label_name = &first_token[..first_token.len() - 1];
                    let label_address = self.program.len();

                    if let Some(prev_fn) = current_fn
                        && let Some(pending_indices) = forward_decls.remove(prev_fn)
                    {
                        let size = self.labels.get(prev_fn).map_or(0, |i| i.max_frame_size);
                        self.patch_forward_calls(prev_fn, &pending_indices, size);
                    }

                    current_fn = Some(label_name);
                    self.labels
                        .entry(label_name)
                        .or_insert_with(|| LabelInfo {
                            address: label_address,
                            max_frame_size: 0,
                        })
                        .address = label_address;

                    if let Some(next_token) = tokens.next() {
                        first_token = next_token;
                    } else {
                        continue;
                    }
                }

                let current_idx = self.program.len();

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

                    "load" => {
                        let n = self.next_arg::<usize>(&mut tokens)?;

                        if let Some(func) = current_fn {
                            let info = self.labels.entry(func).or_default();
                            info.max_frame_size = info.max_frame_size.max(n + 1);
                        }

                        self.program.push(Opcode::Load(n));
                    }

                    "store" => {
                        let n = self.next_arg::<usize>(&mut tokens)?;

                        if let Some(func) = current_fn {
                            let info = self.labels.entry(func).or_default();
                            info.max_frame_size = info.max_frame_size.max(n + 1);
                        }

                        self.program.push(Opcode::Store(n));
                    }

                    "jump" => {
                        let label = tokens.next().ok_or(ParseError::ExpectedValue)?;

                        if let Some(info) = self.labels.get(label) {
                            self.program.push(Opcode::Jump(info.address));
                        } else {
                            forward_decls.entry(label).or_default().push(current_idx);
                            self.program.push(Opcode::Jump(0));
                        }
                    }

                    "jumpifzero" => {
                        let label = tokens.next().ok_or(ParseError::ExpectedValue)?;

                        if let Some(info) = self.labels.get(label) {
                            self.program.push(Opcode::JumpIfZero(info.address));
                        } else {
                            forward_decls.entry(label).or_default().push(current_idx);
                            self.program.push(Opcode::JumpIfZero(0));
                        }
                    }

                    "call" => {
                        let label = tokens.next().ok_or(ParseError::ExpectedValue)?;
                        let num_args = self.next_arg::<usize>(&mut tokens)?;

                        if let Some(func) = current_fn {
                            let info = self.labels.entry(func).or_default();
                            info.max_frame_size = info.max_frame_size.max(num_args);
                        }

                        if let Some(info) = self.labels.get(label) {
                            let final_frame_size = num_args.max(info.max_frame_size);
                            self.program.push(Opcode::Call(
                                info.address,
                                num_args,
                                final_frame_size,
                            ));
                        } else {
                            forward_decls.entry(label).or_default().push(current_idx);
                            self.program.push(Opcode::Call(0, num_args, 0));
                        }
                    }

                    "return" => self.program.push(Opcode::Return),
                    "halt" => self.program.push(Opcode::Halt),
                    other => return Err(ParseError::UnknownInstruction(other)),
                }
            }
        }

        if let Some(last_fn) = current_fn
            && let Some(pending_indices) = forward_decls.remove(last_fn)
        {
            let size = self.labels.get(last_fn).map_or(0, |i| i.max_frame_size);
            self.patch_forward_calls(last_fn, &pending_indices, size);
        }

        if let Some((missing_label, _)) = forward_decls.into_iter().next() {
            return Err(ParseError::UnknownLabel(missing_label));
        }

        Ok(Vm::new(self.program))
    }
}
