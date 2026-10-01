use crate::chunk::{Chunk, OpCode, decode_u24_le};
use crate::value::Value;

pub struct VM<'chunk> {
    chunk: &'chunk Chunk,
    // instruction pointer to the next instruction to exectuce
    ip: usize,
}

impl<'chunk> VM<'chunk> {
    pub fn interpret(chunk: &'chunk Chunk) -> InterpretResult {
        let mut vm = VM { chunk, ip: 0 };
        vm.run()
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
            }
            match opcode {
                OpCode::OP_CONSTANT => {
                    let constant = self.read_constant();
                    println!("{constant}");
                }
                OpCode::OP_CONSTANT_LONG => {
                    let constant = self.read_long_constant();
                    println!("{constant}");
                }
                OpCode::OP_RETURN => return INTERPRET_OK,
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
