use std::fmt::Display;

use color_eyre::eyre::bail;

#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum OpCode {
    OP_RETURN,
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::OP_RETURN => "OP_RETURN",
        };
        write!(f, "{s}")
    }
}

impl TryFrom<u8> for OpCode {
    type Error = color_eyre::Report;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::OP_RETURN),
            _ => bail!("unknown opcode"),
        }
    }
}

pub struct Chunk {
    code: Vec<u8>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Self { code: Vec::new() }
    }

    pub fn code(&self) -> &[u8] {
        self.code.as_slice()
    }

    pub fn write_opcode(&mut self, opcode: OpCode) {
        self.code.push(opcode as u8);
    }

    pub fn write_byte(&mut self, byte: u8) {
        self.code.push(byte);
    }
}
