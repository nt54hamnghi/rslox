use crate::chunk::{Chunk, OpCode, decode_u24_le};
use crate::value::Value;

pub struct VM<'chunk> {
    chunk: &'chunk Chunk,
    // instruction pointer to the next instruction to exectuce
    ip: usize,
    stack: Vec<Value>,
}

impl<'chunk> VM<'chunk> {
    pub fn interpret(chunk: &'chunk Chunk) -> InterpretResult {
        let mut vm = VM {
            chunk,
            ip: 0,
            stack: Vec::new(),
        };
        vm.run()
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Option<Value> {
        self.stack.pop()
    }

    // Reads the byte currently pointed at and advances the
    // instruction pointer
    fn read_byte(&mut self) -> u8 {
        let byte = self.chunk.code()[self.ip];
        self.ip += 1;
        return byte;
    }

    fn read_constant(&mut self) -> Value {
        self.chunk.values()[self.read_byte() as usize]
    }

    fn read_long_constant(&mut self) -> Value {
        let mut bytes = [0u8; 3];
        for i in 0..3 {
            bytes[i] = self.read_byte();
        }
        let idx = decode_u24_le(&bytes);
        self.chunk.values()[idx]
    }

    fn run(&mut self) -> InterpretResult {
        use self::InterpretResult::INTERPRET_OK;

        macro_rules! binary_op {
            ($op:tt) => {{
                let b = self.pop().unwrap();
                let a = self.pop().unwrap();
                self.push(a $op b);
            }};
        }

        loop {
            let byte = self.read_byte();
            // being unable to convert into an opcode indicates
            // bugs in bytecode walking logic
            let opcode = OpCode::try_from(byte).expect("a valid opcode");
            #[cfg(feature = "trace")]
            {
                // self.read_byte advances ip by 1, so ip currently
                // no longer points to the opcode
                self.chunk.disasemble_instruction(self.ip - 1, opcode);

                print!("{:10}", "");
                for v in self.stack.as_slice() {
                    print!("[ {v} ]");
                }
                println!();
            }
            match opcode {
                OpCode::OP_CONSTANT => {
                    let constant = self.read_constant();
                    self.push(constant);
                }
                OpCode::OP_CONSTANT_LONG => {
                    let constant = self.read_long_constant();
                    self.push(constant);
                }
                OpCode::OP_ADD => binary_op!(+),
                OpCode::OP_SUBTRACT => binary_op!(-),
                OpCode::OP_MULTIPLY => binary_op!(*),
                OpCode::OP_DIVIDE => binary_op!(/),
                OpCode::OP_NEGATE => {
                    // TODO: why unwrap here?
                    let value = self.pop().unwrap();
                    self.push(-value);
                }
                OpCode::OP_RETURN => {
                    println!("{}", self.pop().unwrap());
                    return INTERPRET_OK;
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum InterpretResult {
    INTERPRET_OK,
    INTERPRET_COMPILE_ERROR,
    INTERPRET_RUNTIME_ERROR,
}
