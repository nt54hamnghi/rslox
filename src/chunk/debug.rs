use crate::chunk::{Chunk, OpCode};

impl Chunk {
    #[must_use]
    pub fn iter(&self) -> ChunkIterator<'_> {
        ChunkIterator::new(self)
    }

    pub fn disasemble(&self, name: &str) {
        println!("== {name} ==");
        for item in self.iter() {
            self.disasemble_instruction(item);
        }
    }

    pub fn disasemble_instruction(&self, item: ChunkItem<'_>) {
        let ChunkItem {
            offset,
            opcode,
            operands,
        } = item;
        print!("{offset:04} ");
        if offset > 0 && self.get_line(offset) == self.get_line(offset - 1) {
            print!("   | ");
        } else {
            print!("{:04} ", self.get_line(offset));
        }

        match opcode {
            OpCode::OP_CONSTANT => {
                let index = operands.unwrap()[0] as usize;
                let value = &self.constants[index];
                println!("{opcode:<18} {index:>4} {value}");
            }
            OpCode::OP_CONSTANT_LONG => {
                let [low, mid, high] = operands.unwrap().try_into().unwrap();
                let index = u32::from_le_bytes([low, mid, high, 0]) as usize;
                let value = &self.constants[index];
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

#[derive(Debug, Clone, Copy)]
pub struct ChunkItem<'chunk> {
    offset: usize,
    opcode: OpCode,
    operands: Option<&'chunk [u8]>,
}

impl<'chunk> Iterator for ChunkIterator<'chunk> {
    type Item = ChunkItem<'chunk>;

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
                let operands = &code[offset + 1..=offset + 1];
                Some(ChunkItem {
                    offset,
                    opcode,
                    operands: Some(operands),
                })
            }
            OpCode::OP_CONSTANT_LONG => {
                self.offset = offset + 4;
                let operands = &code[offset + 1..=offset + 3];
                Some(ChunkItem {
                    offset,
                    opcode,
                    operands: Some(operands),
                })
            }
            OpCode::OP_RETURN => {
                self.offset = offset + 1;
                Some(ChunkItem {
                    offset,
                    opcode,
                    operands: None,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ChunkItem, OpCode};
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
        let items: Vec<ChunkItem<'_>> = chunk.iter().collect();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].offset, 0);
        assert_eq!(items[0].opcode, OpCode::OP_CONSTANT);
        assert_eq!(items[0].operands, Some(&[0_u8] as &[u8]));
    }

    #[test]
    fn iterator_decodes_constant_long_instruction() {
        let mut chunk = Chunk::new();
        chunk.constants.resize(256, 0.0);
        chunk.write_constant(1.5, 0);
        let items: Vec<ChunkItem<'_>> = chunk.iter().collect();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].offset, 0);
        assert_eq!(items[0].opcode, OpCode::OP_CONSTANT_LONG);
        assert_eq!(items[0].operands, Some(&[0u8, 1, 0] as &[u8]));
    }

    #[test]
    fn iterator_decodes_return_instruction() {
        let mut chunk = Chunk::new();
        chunk.write_opcode(OpCode::OP_RETURN, 0);
        let items: Vec<ChunkItem<'_>> = chunk.iter().collect();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].offset, 0);
        assert_eq!(items[0].opcode, OpCode::OP_RETURN);
        assert_eq!(items[0].operands, None);
    }

    #[test]
    fn iterator_skips_unknown_opcodes() {
        let mut chunk = Chunk::new();
        chunk.write_byte(u8::MAX, 10);
        chunk.write_opcode(OpCode::OP_RETURN, 11);

        let items: Vec<ChunkItem<'_>> = chunk.iter().collect();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].offset, 1);
        assert_eq!(items[0].opcode, OpCode::OP_RETURN);
        assert_eq!(items[0].operands, None);
    }
}
