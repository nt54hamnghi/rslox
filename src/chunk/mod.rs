use std::fmt::Display;

use color_eyre::eyre::bail;

use crate::value::Value;

pub mod debug;

#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OpCode {
    OP_CONSTANT,
    OP_RETURN,
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::OP_CONSTANT => "OP_CONSTANT",
            Self::OP_RETURN => "OP_RETURN",
        };
        write!(f, "{s}")
    }
}

impl TryFrom<u8> for OpCode {
    type Error = color_eyre::Report;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::OP_CONSTANT),
            1 => Ok(Self::OP_RETURN),
            _ => bail!("unknown opcode"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq))]
struct LineStart {
    // offset of the instructions at the start of this line
    offset: usize,
    // the line number
    line: usize,
}

#[derive(Debug)]
pub struct Chunk {
    // stream of bytecode instructions
    code: Vec<u8>,
    // constant table/pool
    constants: Vec<Value>,
    lines: Vec<LineStart>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

impl Chunk {
    /// Creates an empty chunk with no instructions or constants.
    pub fn new() -> Chunk {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }

    pub fn values(&self) -> &[Value] {
        &self.constants
    }

    /// Appends a byte and records the source line where it occurred.
    pub fn write_byte(&mut self, byte: u8, line: usize) {
        self.code.push(byte);

        if !self.lines.is_empty() {
            let last = self.lines.last().unwrap();
            assert!(line >= last.line);
            if last.line == line {
                return;
            }
        }

        self.lines.push(LineStart {
            offset: self.code().len() - 1,
            line,
        });
    }

    /// Appends an opcode and records the source line where it occurred.
    pub fn write_opcode(&mut self, opcode: OpCode, line: usize) {
        self.write_byte(opcode as u8, line);
    }

    /// Adds a constant and writes an `OP_CONSTANT` instruction for it.
    ///
    /// This is a convenience method that assumes the `OP_CONSTANT` instruction and
    /// its operand are on the same source line.
    pub fn write_constant(&mut self, value: Value, line: usize) {
        let const_idx = self.add_constant(value);
        self.write_opcode(OpCode::OP_CONSTANT, line);
        self.write_byte(const_idx as u8, line);
    }

    /// Adds a value to the constant table and returns its index.
    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    fn get_line(&self, offset: usize) -> usize {
        let mut start = 0;
        let mut end = self.lines.len() - 1;
        loop {
            let mid = (end + start) / 2;
            if offset < self.lines[mid].offset {
                end = mid - 1;
            } else if mid == self.lines.len() - 1 || offset < self.lines[mid + 1].offset {
                return self.lines[mid].line;
            } else {
                start = mid + 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::chunk::LineStart;

    use super::{Chunk, OpCode};

    #[rstest]
    #[case(OpCode::OP_CONSTANT, 0, "OP_CONSTANT")]
    #[case(OpCode::OP_RETURN, 1, "OP_RETURN")]
    fn opcode_has_the_expected_byte_and_display_name(
        #[case] opcode: OpCode,
        #[case] byte: u8,
        #[case] name: &str,
    ) {
        assert_eq!(opcode.to_string(), name);
        assert_eq!(opcode as u8, byte);
    }

    #[rstest]
    #[case(0, OpCode::OP_CONSTANT)]
    #[case(1, OpCode::OP_RETURN)]
    fn opcode_can_be_converted_from_its_byte(#[case] byte: u8, #[case] expected: OpCode) {
        assert_eq!(OpCode::try_from(byte).unwrap(), expected);
    }

    #[rstest]
    #[case(u8::MAX)]
    fn invalid_opcode_bytes_are_rejected(#[case] byte: u8) {
        assert!(OpCode::try_from(byte).is_err());
    }

    #[test]
    fn new_chunk_is_empty() {
        let chunk = Chunk::new();

        assert!(chunk.code().is_empty());
        assert!(chunk.values().is_empty());
        assert!(chunk.lines.is_empty());
    }

    #[test]
    fn default_chunk_is_empty() {
        let chunk = Chunk::default();

        assert!(chunk.code().is_empty());
        assert!(chunk.values().is_empty());
        assert!(chunk.lines.is_empty());
    }

    #[rstest]
    #[case(0, 1)]
    #[case(u8::MAX, 42)]
    fn write_byte_appends_a_byte_and_its_line(#[case] byte: u8, #[case] line: usize) {
        let mut chunk = Chunk::new();

        chunk.write_byte(byte, line);

        assert_eq!(chunk.code(), [byte]);
        assert_eq!(chunk.lines, [LineStart { offset: 0, line }]);
    }

    #[test]
    fn write_byte_records_starts_of_consecutive_line_runs() {
        let mut chunk = Chunk::new();

        chunk.write_byte(0, 10);
        chunk.write_byte(1, 10);
        chunk.write_byte(2, 11);
        chunk.write_byte(3, 11);

        assert_eq!(chunk.code(), [0, 1, 2, 3]);
        assert_eq!(
            chunk.lines,
            [
                LineStart {
                    offset: 0,
                    line: 10
                },
                LineStart {
                    offset: 2,
                    line: 11
                },
            ]
        );
    }

    #[rstest]
    #[case(0, 0)]
    #[case(1, 0)]
    #[case(2, 1)]
    #[case(3, 1)]
    #[case(4, 2)]
    fn get_line_returns_the_line_for_each_offset(#[case] offset: usize, #[case] expected: usize) {
        let mut chunk = Chunk::new();
        chunk.write_byte(0, 0);
        chunk.write_byte(1, 0);
        chunk.write_byte(2, 1);
        chunk.write_byte(3, 1);
        chunk.write_byte(4, 2);

        assert_eq!(chunk.get_line(offset), expected);
    }

    #[test]
    #[should_panic]
    fn write_byte_panics_when_a_line_decreases() {
        let mut chunk = Chunk::new();

        chunk.write_byte(0, 11);
        chunk.write_byte(1, 10);
    }

    #[rstest]
    #[case(OpCode::OP_CONSTANT, 1)]
    #[case(OpCode::OP_RETURN, 42)]
    fn write_opcode_appends_an_opcode_and_its_line(#[case] opcode: OpCode, #[case] line: usize) {
        let mut chunk = Chunk::new();

        chunk.write_opcode(opcode, line);

        assert_eq!(chunk.code(), [opcode as u8]);
        assert_eq!(chunk.lines, [LineStart { offset: 0, line }]);
    }

    #[test]
    fn add_constant_returns_its_index() {
        let mut chunk = Chunk::new();

        assert_eq!(chunk.add_constant(1.5), 0);
        assert_eq!(chunk.add_constant(-2.0), 1);
        assert_eq!(chunk.values(), [1.5, -2.0]);
    }

    #[test]
    fn write_constant_adds_the_value_and_its_instruction() {
        let mut chunk = Chunk::new();
        chunk.write_constant(8.5, 0);

        assert_eq!(chunk.code(), [OpCode::OP_CONSTANT as u8, 0]);
        assert_eq!(chunk.values(), [8.5]);
        assert_eq!(chunk.lines, [LineStart { offset: 0, line: 0 }]);
    }
}
