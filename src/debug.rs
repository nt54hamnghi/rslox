use crate::chunk::{Chunk, OpCode};

impl Chunk {
    pub fn disasemble(&self, name: &str) {
        println!("== {name} ==");
        for (offset, instruction) in Disassembler::new(self) {
            print!("{offset:04} ");
            println!("{instruction}");
        }
    }
}

struct Disassembler<'chunk> {
    chunk: &'chunk Chunk,
    offset: usize,
}

impl<'chunk> Disassembler<'chunk> {
    fn new(chunk: &'chunk Chunk) -> Self {
        Self { chunk, offset: 0 }
    }
}

impl<'chunk> Iterator for Disassembler<'chunk> {
    type Item = (usize, String); // (offset, string repr of the instruction)

    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.offset;
        let byte = *self.chunk.code().get(offset)?;

        match byte.try_into() {
            Ok(opcode) => match opcode {
                OpCode::OP_RETURN => {
                    self.offset = offset + 1;
                    Some((offset, opcode.to_string()))
                }
            },
            Err(_) => {
                println!("Unknown opcode {byte}");
                self.offset = offset + 1;
                self.next()
            }
        }
    }
}
