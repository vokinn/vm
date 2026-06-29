use crate::{errors::VmError, instruction::Opcode};

pub const MAX_LOCALS: usize = 64;

pub struct Vm {
    stack: Vec<i64>,
    variables: Vec<i64>,
    program: Vec<Opcode>,
    ip: usize,
}

impl Vm {
    pub fn new(program: Vec<Opcode>) -> Self {
        Self {
            stack: vec![],
            variables: vec![0; MAX_LOCALS],
            program,
            ip: 0,
        }
    }

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
