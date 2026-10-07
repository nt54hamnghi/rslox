use crate::chunk::{Chunk, OpCode, decode_u24_le};
use crate::compiler::compile;
use crate::errors::Error;
use crate::stack::Stack;
use crate::value::Value;

pub struct VM<'chunk> {
    chunk: &'chunk Chunk,
    // instruction pointer to the next instruction to exectuce
    ip: usize,
    stack: Stack<Value, 512>,
}

impl<'chunk> VM<'chunk> {
    pub fn new(chunk: &'chunk Chunk, ip: usize) -> VM<'chunk> {
        VM {
            chunk,
            ip,
            stack: Stack::new(),
        }
    }

    pub fn interpret(source: &str) -> Result<(), Error> {
        compile(source);
        Ok(())
    }

    /// Reads the byte currently pointed at and advances the
    /// instruction pointer
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

    fn run(&mut self) -> Result<(), Error> {
        macro_rules! binary_op {
            ($op:tt) => {{
                // A binary operation expects its two operands on the stack.
                // If the stack contains fewer than two values, something went
                // wrong during compilation: the compiler should have rejected
                // the invalid expression before the VM started executing.
                let b = self.stack.pop().unwrap();
                let a = self.stack.pop().unwrap();
                self.stack.push(a $op b).unwrap();
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
                self.debug_stack();
            }

            match opcode {
                OpCode::OP_CONSTANT => {
                    let constant = self.read_constant();
                    self.stack.push(constant).unwrap();
                }
                OpCode::OP_CONSTANT_LONG => {
                    let constant = self.read_long_constant();
                    self.stack.push(constant).unwrap();
                }
                OpCode::OP_ADD => binary_op!(+),
                OpCode::OP_SUBTRACT => binary_op!(-),
                OpCode::OP_MULTIPLY => binary_op!(*),
                OpCode::OP_DIVIDE => binary_op!(/),
                OpCode::OP_NEGATE => {
                    // OP_NEGATE expects a value on the stack.
                    // If the stack is empty, something went wrong during compilation:
                    // the compiler should have rejected the invalid expression before
                    // the VM started executing.
                    let value = self.stack.pop().unwrap();
                    self.stack.push(-value).unwrap();
                }
                OpCode::OP_RETURN => {
                    println!("{}", self.stack.pop().unwrap());
                    return Ok(());
                }
            }
        }
    }

    #[allow(unused)]
    fn debug_stack(&self) {
        print!("{:10}", "");
        for v in self.stack.as_slice() {
            print!("[ {v} ]");
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::VM;
    use crate::{chunk::Chunk, stack::Stack};

    #[rstest]
    #[case(0, 42)]
    #[case(1, 0)]
    #[case(2, 255)]
    fn read_byte_reads_from_the_current_position_and_advances(
        #[case] position: usize,
        #[case] expected: u8,
    ) {
        let mut chunk = Chunk::new();
        for byte in [42, 0, 255] {
            chunk.write_byte(byte, 1);
        }
        let mut vm = VM::new(&chunk, 0);
        vm.ip = position;

        assert_eq!(vm.read_byte(), expected);
        assert_eq!(vm.ip, position + 1);
    }

    #[rstest]
    #[case(0, 1.5)]
    #[case(1, -2.0)]
    fn read_constant_returns_the_indexed_value(#[case] index: u8, #[case] expected: f64) {
        let mut chunk = Chunk::new();
        chunk.add_constant(1.5);
        chunk.add_constant(-2.0);
        chunk.write_byte(index, 1);
        let mut vm = VM::new(&chunk, 0);

        assert_eq!(vm.read_constant(), expected);
        assert_eq!(vm.ip, 1);
    }

    #[rstest]
    #[case(0, 1.5)]
    #[case(1, -2.0)]
    fn read_long_constant_returns_the_indexed_value(#[case] index: u8, #[case] expected: f64) {
        let mut chunk = Chunk::new();
        chunk.add_constant(1.5);
        chunk.add_constant(-2.0);
        chunk.write_byte(index, 1);
        chunk.write_byte(0, 1);
        chunk.write_byte(0, 1);
        let mut vm = VM::new(&chunk, 0);

        assert_eq!(vm.read_long_constant(), expected);
        assert_eq!(vm.ip, 3);
    }
}
