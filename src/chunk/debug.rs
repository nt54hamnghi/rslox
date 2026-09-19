use crate::chunk::{Chunk, OpCode};

impl Chunk {
    pub fn iter(&self) -> ChunkIterator<'_> {
        ChunkIterator::new(self)
    }

    pub fn disasemble(&self, name: &str) {
        println!("== {name} ==");
        for ChunkItem {
            offset,
            opcode,
            operands,
        } in self.iter()
        {
            print!("{offset:04} ");
            if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
                print!("   | ");
            } else {
                print!("{:04} ", self.lines[offset]);
            }

            match opcode {
                OpCode::OP_CONSTANT => {
                    let index = operands.unwrap()[0] as usize;
                    let value = &self.constants[index];
                    println!("{opcode:<16} {index:>4} {value}'");
                }
                OpCode::OP_RETURN => {
                    println!("{opcode}")
                }
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

#[derive(Debug)]
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
        match byte.try_into() {
            Ok(opcode) => match opcode {
                OpCode::OP_CONSTANT => {
                    self.offset = offset + 2;
                    let operands = &code[offset + 1..=offset + 1];
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
            },
            Err(_) => {
                eprintln!("Unknown opcode {byte}");
                self.offset = offset + 1;
                self.next()
            }
        }
    }
}
