use crate::{errors::VmError, instruction::Opcode};

const MAX_LOCALS: usize = 64;

struct Frame {
    return_addr: usize,
    locals: Vec<i64>,
}

impl Frame {
    fn new(return_addr: usize, capacity: usize) -> Self {
        Self {
            return_addr,
            locals: Vec::with_capacity(capacity),
        }
    }
}

pub struct Vm {
    operand_stack: Vec<i64>,
    call_stack: Vec<Frame>,
    program: Vec<Opcode>,
    ip: usize,
}

impl Vm {
    pub(crate) fn new(program: Vec<Opcode>) -> Self {
        Self {
            operand_stack: vec![],
            call_stack: vec![Frame::new(0, MAX_LOCALS)],
            program,
            ip: 0,
        }
    }

    fn call(&mut self, opcode: Opcode) -> Result<(), VmError> {
        match opcode {
            Opcode::Push(n) => self.operand_stack.push(n),
            Opcode::Add => {
                let a = self.operand_stack.pop().ok_or(VmError::StackEmpty)?;
                let b = self.operand_stack.pop().ok_or(VmError::StackEmpty)?;

                self.operand_stack.push(a + b);
            }

            Opcode::Subtract => {
                let a = self.operand_stack.pop().ok_or(VmError::StackEmpty)?;
                let b = self.operand_stack.pop().ok_or(VmError::StackEmpty)?;

                self.operand_stack.push(b - a);
            }

            Opcode::Multiply => {
                let a = self.operand_stack.pop().ok_or(VmError::StackEmpty)?;
                let b = self.operand_stack.pop().ok_or(VmError::StackEmpty)?;

                self.operand_stack.push(a * b);
            }

            Opcode::Divide => {
                let a = self.operand_stack.pop().ok_or(VmError::StackEmpty)?;
                let b = self.operand_stack.pop().ok_or(VmError::StackEmpty)?;

                if a == 0 {
                    return Err(VmError::DivByZero);
                }

                self.operand_stack.push(b / a);
            }

            Opcode::Print => {
                let top = self.operand_stack.pop().ok_or(VmError::StackEmpty)?;
                println!("{}", top);
            }

            Opcode::Debug => {
                println!("{:?}", self.operand_stack);
            }

            Opcode::Duplicate => {
                let top = self.operand_stack.last().ok_or(VmError::StackEmpty)?;
                self.operand_stack.push(*top);
            }

            Opcode::Store(n) => {
                let top = self.operand_stack.pop().ok_or(VmError::StackEmpty)?;
                self.call_stack
                    .last_mut()
                    .ok_or(VmError::CallStackEmpty)?
                    .locals[n] = top;
            }

            Opcode::Load(n) => {
                let top = self
                    .call_stack
                    .last_mut()
                    .ok_or(VmError::CallStackEmpty)?
                    .locals
                    .get(n)
                    .ok_or(VmError::InvalidVariable)?;

                self.operand_stack.push(*top);
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
                    if self.operand_stack.pop().ok_or(VmError::StackEmpty)? == 0 {
                        self.ip = n;
                    };
                }

                Opcode::Call(fn_addr, num_args) => {
                    let mut frame = Frame::new(self.ip, num_args);
                    frame.locals.resize(num_args, 0);

                    for i in (0..num_args).rev() {
                        frame.locals[i] = self.operand_stack.pop().ok_or(VmError::StackEmpty)?;
                    }

                    self.call_stack.push(frame);
                    self.ip = fn_addr;
                }

                Opcode::Return => {
                    let frame = self.call_stack.pop().ok_or(VmError::CallStackEmpty)?;
                    self.ip = frame.return_addr;
                }

                Opcode::Halt => break,
                _ => self.call(opcode)?,
            }
        }

        Ok(())
    }
}
