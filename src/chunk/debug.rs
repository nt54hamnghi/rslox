use crate::chunk::{Chunk, OpCode, decode_u24_le};

impl Chunk {
    #[must_use]
    pub fn iter(&self) -> ChunkIterator<'_> {
        ChunkIterator::new(self)
    }

    pub fn disasemble(&self, name: &str) {
        println!("== {name} ==");
        for (offset, opcode) in self.iter() {
            self.disasemble_instruction(offset, opcode);
        }
    }

    pub fn disasemble_instruction(&self, offset: usize, opcode: OpCode) {
        print!("{offset:04} ");
        if offset > 0 && self.get_line(offset) == self.get_line(offset - 1) {
            print!("   | ");
        } else {
            print!("{:04} ", self.get_line(offset));
        }

        match opcode {
            OpCode::OP_CONSTANT => {
                let index = self.code[offset + 1] as usize;
                let value = self.constants[index];
                println!("{opcode:<18} {index:>4} {value}");
            }
            OpCode::OP_CONSTANT_LONG => {
                let bytes = &self.code[offset + 1..=offset + 3];
                let index = decode_u24_le(bytes);
                let value = self.constants[index];
                println!("{opcode:<18} {index:>4} {value}");
            }
            OpCode::OP_RETURN => {
                println!("{opcode}");
            }
        }
    }
}

#[derive(Debug)]
pub struct ChunkIterator<'chunk> {
    chunk: &'chunk Chunk,
    offset: usize,
}

impl<'chunk> ChunkIterator<'chunk> {
    fn new(chunk: &'chunk Chunk) -> Self {
        Self { chunk, offset: 0 }
    }
}

impl<'chunk> Iterator for ChunkIterator<'chunk> {
    type Item = (usize, OpCode); // (offset, opcode)

    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.offset;
        let code = &self.chunk.code;

        let byte = *code.get(offset)?;
        let Ok(opcode) = byte.try_into() else {
            eprintln!("Unknown opcode {byte}");
            self.offset = offset + 1;
            return self.next();
        };

        match opcode {
            OpCode::OP_CONSTANT => {
                self.offset = offset + 2;
                Some((offset, opcode))
            }
            OpCode::OP_CONSTANT_LONG => {
                self.offset = offset + 4;
                Some((offset, opcode))
            }
            OpCode::OP_RETURN => {
                self.offset = offset + 1;
                Some((offset, opcode))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OpCode;
    use crate::chunk::Chunk;

    #[test]
    fn iterator_is_empty_for_an_empty_chunk() {
        let chunk = Chunk::new();

        assert!(chunk.iter().next().is_none());
    }

    #[test]
    fn iterator_decodes_constant_instruction() {
        let mut chunk = Chunk::new();
        chunk.write_constant(1.5, 0);
        let items: Vec<(usize, OpCode)> = chunk.iter().collect();

        assert_eq!(items.len(), 1);
        let (offset, opcode) = items[0];
        assert_eq!(offset, 0);
        assert_eq!(opcode, OpCode::OP_CONSTANT);
    }

    #[test]
    fn iterator_decodes_constant_long_instruction() {
        let mut chunk = Chunk::new();
        chunk.constants.resize(256, 0.0);
        chunk.write_constant(1.5, 0);
        let items: Vec<(usize, OpCode)> = chunk.iter().collect();

        assert_eq!(items.len(), 1);
        let (offset, opcode) = items[0];
        assert_eq!(offset, 0);
        assert_eq!(opcode, OpCode::OP_CONSTANT_LONG);
    }

    #[test]
    fn iterator_decodes_return_instruction() {
        let mut chunk = Chunk::new();
        chunk.write_opcode(OpCode::OP_RETURN, 0);
        let items: Vec<(usize, OpCode)> = chunk.iter().collect();

        assert_eq!(items.len(), 1);
        let (offset, opcode) = items[0];
        assert_eq!(offset, 0);
        assert_eq!(opcode, OpCode::OP_RETURN);
    }

    #[test]
    fn iterator_skips_unknown_opcodes() {
        let mut chunk = Chunk::new();
        chunk.write_byte(u8::MAX, 10);
        chunk.write_opcode(OpCode::OP_RETURN, 11);

        let items: Vec<(usize, OpCode)> = chunk.iter().collect();

        assert_eq!(items.len(), 1);
        let (offset, opcode) = items[0];
        assert_eq!(offset, 1);
        assert_eq!(opcode, OpCode::OP_RETURN);
    }
}
