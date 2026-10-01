use std::fmt::Display;

use color_eyre::eyre::bail;

use crate::value::Value;

pub mod debug;

#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OpCode {
    // Load a constant using a 8-bit index.
    OP_CONSTANT,
    // Load a constant using a 24-bit index.
    OP_CONSTANT_LONG,
    OP_RETURN,
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::OP_CONSTANT => "OP_CONSTANT",
            Self::OP_CONSTANT_LONG => "OP_CONSTANT_LONG",
            Self::OP_RETURN => "OP_RETURN",
        };
        f.pad(s)
    }
}

impl TryFrom<u8> for OpCode {
    type Error = color_eyre::Report;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::OP_CONSTANT),
            1 => Ok(Self::OP_CONSTANT_LONG),
            2 => Ok(Self::OP_RETURN),
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

    /// Adds a value to the constant table and writes an instruction to load it.
    ///
    /// If the value's index fits in 8 bits, it is encoded as a one-byte operand
    /// using `OP_CONSTANT`. If it fits in 24 bits, it is encoded as a three-byte
    /// little-endian operand using `OP_CONSTANT_LONG`.
    ///
    /// Records `line` for the opcode and all operand bytes.
    ///
    /// # Panics
    ///
    /// Panics if the index into the constant table does not fit in 24 bits or if
    /// `line` is less than the last recorded source line.
    pub fn write_constant(&mut self, value: Value, line: usize) {
        let idx = self.add_constant(value);

        if let Ok(b) = u8::try_from(idx) {
            self.write_opcode(OpCode::OP_CONSTANT, line);
            self.write_byte(b, line);
            return;
        }

        assert!(idx < 2usize.pow(24));
        self.write_opcode(OpCode::OP_CONSTANT_LONG, line);
        let bytes = idx.to_le_bytes();
        for b in &bytes[0..3] {
            self.write_byte(*b, line);
        }
    }

    /// Adds a value to the constant table and returns its index.
    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    /// Returns the source line associated with the byte at `offset` in the
    /// chunk's bytecode.
    ///
    /// # Panics
    ///
    /// Panics if `offset` is greater than or equal to the bytecode length.
    fn get_line(&self, offset: usize) -> usize {
        assert!(offset < self.code().len());

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

/// Reads a 24-bit unsigned integer from three little-endian bytes.
///
/// # Panics
///
/// Panics if `bytes` does not contain exactly three bytes.
pub fn decode_u24_le(bytes: &[u8]) -> usize {
    let [low, mid, high] = bytes.try_into().expect("bytes to have length of 3");
    u32::from_le_bytes([low, mid, high, 0]) as usize
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::chunk::LineStart;

    use super::{Chunk, OpCode};

    #[rstest]
    #[case(OpCode::OP_CONSTANT, 0, "OP_CONSTANT")]
    #[case(OpCode::OP_CONSTANT_LONG, 1, "OP_CONSTANT_LONG")]
    #[case(OpCode::OP_RETURN, 2, "OP_RETURN")]
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
    #[case(1, OpCode::OP_CONSTANT_LONG)]
    #[case(2, OpCode::OP_RETURN)]
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

        assert!(chunk.code.is_empty());
        assert!(chunk.constants.is_empty());
        assert!(chunk.lines.is_empty());
    }

    #[test]
    fn default_chunk_is_empty() {
        let chunk = Chunk::default();

        assert!(chunk.code.is_empty());
        assert!(chunk.constants.is_empty());
        assert!(chunk.lines.is_empty());
    }

    #[rstest]
    #[case(0, 1)]
    #[case(u8::MAX, 42)]
    fn write_byte_appends_a_byte_and_its_line(#[case] byte: u8, #[case] line: usize) {
        let mut chunk = Chunk::new();

        chunk.write_byte(byte, line);

        assert_eq!(chunk.code, [byte]);
        assert_eq!(chunk.lines, [LineStart { offset: 0, line }]);
    }

    #[test]
    fn write_byte_records_starts_of_consecutive_line_runs() {
        let mut chunk = Chunk::new();

        chunk.write_byte(0, 10);
        chunk.write_byte(1, 10);
        chunk.write_byte(2, 11);
        chunk.write_byte(3, 11);

        assert_eq!(chunk.code, [0, 1, 2, 3]);
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

        assert_eq!(chunk.code, [opcode as u8]);
        assert_eq!(chunk.lines, [LineStart { offset: 0, line }]);
    }

    #[test]
    fn add_constant_returns_its_index() {
        let mut chunk = Chunk::new();

        assert_eq!(chunk.add_constant(1.5), 0);
        assert_eq!(chunk.add_constant(-2.0), 1);
        assert_eq!(chunk.constants, [1.5, -2.0]);
    }

    #[test]
    fn write_constant_adds_the_value_and_its_instruction() {
        let mut chunk = Chunk::new();
        chunk.write_constant(8.5, 0);

        assert_eq!(chunk.code, [OpCode::OP_CONSTANT as u8, 0]);
        assert_eq!(chunk.constants, [8.5]);
        assert_eq!(chunk.lines, [LineStart { offset: 0, line: 0 }]);
    }

    #[rstest]
    #[case(255, OpCode::OP_CONSTANT, vec![0xff])]
    #[case(256, OpCode::OP_CONSTANT_LONG, vec![0x00, 0x01, 0x00])]
    #[case(0x01_02_03, OpCode::OP_CONSTANT_LONG, vec![0x03, 0x02, 0x01])]
    #[case(0xff_ff_ff, OpCode::OP_CONSTANT_LONG, vec![0xff, 0xff, 0xff])]
    fn write_constant_encodes_the_index(
        #[case] index: usize,
        #[case] opcode: OpCode,
        #[case] operands: Vec<u8>,
    ) {
        let mut chunk = Chunk::new();
        chunk.constants.resize(index, 0.0);

        chunk.write_constant(8.5, 42);

        assert_eq!(chunk.constants.len(), index + 1);
        assert_eq!(chunk.constants[index], 8.5);

        let mut expected = vec![opcode as u8];
        expected.extend(operands);
        assert_eq!(chunk.code, expected);
        assert_eq!(
            chunk.lines,
            [LineStart {
                offset: 0,
                line: 42
            }]
        );
    }

    #[test]
    #[should_panic(expected = "assertion failed: idx < 2usize.pow(24)")]
    fn write_constant_rejects_an_index_larger_than_24_bits() {
        let mut chunk = Chunk::new();
        chunk.constants.resize(0x01_00_00_00, 0.0);

        chunk.write_constant(8.5, 42);
    }
}
