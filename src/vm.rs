use color_eyre::eyre::eyre;

use crate::chunk::{Chunk, OpCode, decode_u24_le};
use crate::compiler::compile;
use crate::errors::Error;
use crate::stack::Stack;
use crate::value::Value;

pub struct VM<'chunk> {
    chunk: &'chunk Chunk,
    // instruction pointer to the next instruction to execute
    ip: usize,
    stack: Stack<Value, 512>,
}

impl<'chunk> VM<'chunk> {
    pub fn new(chunk: &'chunk Chunk) -> VM<'chunk> {
        VM {
            chunk,
            ip: 0,
            stack: Stack::new(),
        }
    }

    pub fn interpret(source: &str) -> Result<(), Error> {
        let mut chunk = Chunk::new();
        compile(source, &mut chunk)?;
        let mut vm = VM::new(&chunk);
        vm.run()
    }

    /// Reads the byte currently pointed at and advances the
    /// instruction pointer
    fn read_byte(&mut self) -> u8 {
        let byte = self.chunk.code()[self.ip];
        self.ip += 1;
        byte
    }

    /// Reads the constant index currently pointed at, advances the
    /// instruction pointer, and returns the indexed value
    fn read_constant(&mut self) -> Value {
        self.chunk.values()[self.read_byte() as usize]
    }

    /// Reads the three-byte constant index currently pointed at, advances the
    /// instruction pointer, and returns the indexed value
    fn read_constant_24(&mut self) -> Value {
        let mut bytes = [0u8; 3];
        for b in &mut bytes {
            *b = self.read_byte();
        }
        let idx = decode_u24_le(&bytes);
        self.chunk.values()[idx]
    }

    fn run(&mut self) -> Result<(), Error> {
        loop {
            let byte = self.read_byte();
            let opcode = OpCode::try_from(byte).expect("a valid opcode");

            #[cfg(feature = "trace")]
            {
                // `self.read_byte` advances ip by 1, so ip is after the opcode.
                self.chunk.disasemble_instruction(self.ip - 1, opcode);
                self.debug_stack();
            }

            self.dispatch(opcode).map_err(|e| {
                self.stack.clear();
                Error::Runtime(e)
            })?;

            if self.ip >= self.chunk.code().len() {
                break Ok(());
            }
        }
    }

    fn dispatch(&mut self, opcode: OpCode) -> color_eyre::Result<()> {
        macro_rules! binary_op {
            ($op:tt) => {{
                // A binary operation expects its two operands on the stack.
                // If the stack contains fewer than two values, something went
                // wrong during compilation: the compiler should have rejected
                // the invalid expression before the VM started executing.
                let b = self.stack.pop().unwrap();
                let a = self.stack.pop().unwrap();
                let (Value::Number(a), Value::Number(b)) = (a, b) else {
                    return Err(self.error("both operands must be numbers"));
                };
                self.stack.push((a $op b).into()).unwrap();
            }};
        }

        match opcode {
            OpCode::OP_NIL => self.stack.push(Value::Nil)?,
            OpCode::OP_TRUE => self.stack.push(true.into())?,
            OpCode::OP_FALSE => self.stack.push(false.into())?,
            OpCode::OP_CONSTANT => {
                let constant = self.read_constant();
                self.stack.push(constant)?;
            }
            OpCode::OP_CONSTANT_24 => {
                let constant = self.read_constant_24();
                self.stack.push(constant)?;
            }
            OpCode::OP_EQUAL => {
                let b = self.stack.pop().unwrap();
                let a = self.stack.pop().unwrap();
                self.stack.push((a == b).into()).unwrap();
            }
            OpCode::OP_GREATER => binary_op!(>),
            OpCode::OP_LESS => binary_op!(<),
            OpCode::OP_ADD => binary_op!(+),
            OpCode::OP_SUBTRACT => binary_op!(-),
            OpCode::OP_MULTIPLY => binary_op!(*),
            OpCode::OP_DIVIDE => binary_op!(/),
            OpCode::OP_NOT => {
                let value = self.stack.pop().unwrap();
                self.stack.push(value.is_falsey().into()).unwrap();
            }
            OpCode::OP_NEGATE => {
                let Value::Number(n) = self.stack.pop().unwrap() else {
                    return Err(self.error("operand must be a number"));
                };
                self.stack.push((-n).into()).unwrap();
            }
            OpCode::OP_RETURN => {
                println!("{}", self.stack.pop().unwrap());
            }
        };

        Ok(())
    }

    fn error(&mut self, msg: &str) -> color_eyre::Report {
        let line = self.chunk.get_line(self.ip - 1);
        eyre!("[line {line}] error: {msg}")
    }

    #[cfg(feature = "trace")]
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
    use crate::chunk::{Chunk, OpCode};
    use crate::errors::Error;
    use crate::value::Value;

    #[rstest]
    #[case(0, 42)]
    #[case(1, 0)]
    #[case(2, 255)]
    fn read_byte_advances_position(#[case] position: usize, #[case] expected: u8) {
        let mut chunk = Chunk::new();
        for byte in [42, 0, 255] {
            chunk.write_byte(byte, 1);
        }
        let mut vm = VM::new(&chunk);
        vm.ip = position;

        assert_eq!(vm.read_byte(), expected);
        assert_eq!(vm.ip, position + 1);
    }

    #[rstest]
    #[case(0, 1.5)]
    #[case(1, -2.0)]
    fn read_constant_returns_value(#[case] index: u8, #[case] expected: f64) {
        let mut chunk = Chunk::new();
        chunk.add_constant((1.5).into());
        chunk.add_constant((-2.0).into());
        chunk.write_byte(index, 1);
        let mut vm = VM::new(&chunk);

        assert_eq!(vm.read_constant(), expected.into());
        assert_eq!(vm.ip, 1);
    }

    #[rstest]
    #[case(0, 1.5)]
    #[case(1, -2.0)]
    fn read_constant_24_returns_value(#[case] index: u8, #[case] expected: f64) {
        let mut chunk = Chunk::new();
        chunk.add_constant((1.5).into());
        chunk.add_constant((-2.0).into());
        chunk.write_byte(index, 1);
        chunk.write_byte(0, 1);
        chunk.write_byte(0, 1);
        let mut vm = VM::new(&chunk);

        assert_eq!(vm.read_constant_24(), expected.into());
        assert_eq!(vm.ip, 3);
    }

    #[rstest]
    #[case::nil(OpCode::OP_NIL, Value::Nil)]
    #[case::true_literal(OpCode::OP_TRUE, true.into())]
    #[case::false_literal(OpCode::OP_FALSE, false.into())]
    fn op_literal_pushes_value(#[case] opcode: OpCode, #[case] expected: Value) {
        let mut chunk = Chunk::new();
        chunk.write_opcode(opcode, 1);

        let mut vm = VM::new(&chunk);
        vm.run().unwrap();

        assert_eq!(vm.stack.as_slice(), &[expected]);
    }

    #[rstest]
    #[case(Value::Nil)]
    #[case(true.into())]
    #[case(false.into())]
    #[case(42.5.into())]
    fn op_constant_pushes_constant(#[case] value: Value) {
        let mut chunk = Chunk::new();
        chunk.write_constant(value, 1);

        let mut vm = VM::new(&chunk);
        vm.run().unwrap();

        assert_eq!(vm.stack.as_slice(), &[value]);
    }

    #[test]
    fn op_constant_24_pushes_constant() {
        let mut chunk = Chunk::new();
        for _ in 0..256 {
            chunk.add_constant(Value::Nil);
        }

        chunk.write_constant(true.into(), 1);

        let mut vm = VM::new(&chunk);
        vm.run().unwrap();

        assert_eq!(vm.stack.as_slice(), &[Value::from(true)]);
    }

    #[rstest]
    #[case::add(OpCode::OP_ADD, 10.0)]
    #[case::subtract(OpCode::OP_SUBTRACT, 6.0)]
    #[case::multiply(OpCode::OP_MULTIPLY, 16.0)]
    #[case::divide(OpCode::OP_DIVIDE, 4.0)]
    fn op_binary_pushes_result(#[case] opcode: OpCode, #[case] expected: f64) {
        let mut chunk = Chunk::new();
        chunk.write_opcode(opcode, 1);

        let mut vm = VM::new(&chunk);
        vm.stack.push(8.0.into()).unwrap();
        vm.stack.push(2.0.into()).unwrap();
        vm.run().unwrap();

        assert_eq!(vm.stack.as_slice(), &[Value::from(expected)]);
    }

    #[rstest]
    #[case::nil(Value::Nil, Value::Nil, true)]
    #[case::true_boolean(true.into(), true.into(), true)]
    #[case::false_boolean(false.into(), false.into(), true)]
    #[case::different_booleans(true.into(), false.into(), false)]
    #[case::equal_numbers(2.0.into(), 2.0.into(), true)]
    #[case::different_numbers(2.0.into(), 8.0.into(), false)]
    #[case::signed_zero(0.0.into(), (-0.0).into(), true)]
    #[case::nil_and_false(Value::Nil, false.into(), false)]
    #[case::nil_and_number(Value::Nil, 0.0.into(), false)]
    #[case::boolean_and_number(true.into(), 1.0.into(), false)]
    fn op_equal_pushes_boolean(#[case] left: Value, #[case] right: Value, #[case] expected: bool) {
        let mut chunk = Chunk::new();
        chunk.write_opcode(OpCode::OP_EQUAL, 1);

        let mut vm = VM::new(&chunk);
        vm.stack.push(left).unwrap();
        vm.stack.push(right).unwrap();
        vm.run().unwrap();

        assert_eq!(vm.stack.as_slice(), &[expected.into()]);
    }

    #[rstest]
    #[case::greater_true(OpCode::OP_GREATER, 8.0, 2.0, true)]
    #[case::greater_false(OpCode::OP_GREATER, 2.0, 8.0, false)]
    #[case::greater_equal(OpCode::OP_GREATER, 2.0, 2.0, false)]
    #[case::less_true(OpCode::OP_LESS, 2.0, 8.0, true)]
    #[case::less_false(OpCode::OP_LESS, 8.0, 2.0, false)]
    #[case::less_equal(OpCode::OP_LESS, 2.0, 2.0, false)]
    fn op_comparison_pushes_boolean(
        #[case] opcode: OpCode,
        #[case] left: f64,
        #[case] right: f64,
        #[case] expected: bool,
    ) {
        let mut chunk = Chunk::new();
        chunk.write_opcode(opcode, 1);

        let mut vm = VM::new(&chunk);
        vm.stack.push(left.into()).unwrap();
        vm.stack.push(right.into()).unwrap();
        vm.run().unwrap();

        assert_eq!(vm.stack.as_slice(), &[expected.into()]);
    }

    #[rstest]
    #[case::nil(Value::Nil, true)]
    #[case::false_boolean(false.into(), true)]
    #[case::true_boolean(true.into(), false)]
    #[case::zero(0.0.into(), false)]
    #[case::negative((-1.0).into(), false)]
    #[case::positive(1.0.into(), false)]
    fn op_not_negates_truthiness(#[case] operand: Value, #[case] expected: bool) {
        let mut chunk = Chunk::new();
        chunk.write_opcode(OpCode::OP_NOT, 1);

        let mut vm = VM::new(&chunk);
        vm.stack.push(operand).unwrap();
        vm.run().unwrap();

        assert_eq!(vm.stack.as_slice(), &[expected.into()]);
    }

    #[rstest]
    #[case(8.0, -8.0)]
    #[case(-8.0, 8.0)]
    #[case(0.0, -0.0)]
    fn op_negate_pushes_result(#[case] operand: f64, #[case] expected: f64) {
        let mut chunk = Chunk::new();
        chunk.write_opcode(OpCode::OP_NEGATE, 1);

        let mut vm = VM::new(&chunk);
        vm.stack.push(operand.into()).unwrap();

        vm.run().unwrap();
        assert_eq!(vm.stack.as_slice(), &[Value::from(expected)]);
    }

    #[rstest]
    #[case(Value::Nil)]
    #[case(true.into())]
    #[case(false.into())]
    fn op_negate_rejects_non_number(#[case] operand: Value) {
        let mut chunk = Chunk::new();
        chunk.write_opcode(OpCode::OP_NEGATE, 1);

        let mut vm = VM::new(&chunk);
        // keep a value below the operand to ensure that
        // the stack is actually cleared after an error,
        // rather than merely emptied when the operand is popped.
        vm.stack.push(Value::Nil).unwrap();
        vm.stack.push(operand).unwrap();

        let res = vm.run();
        assert!(matches!(&res, Err(Error::Runtime(_))));

        let err = res.unwrap_err();
        assert_eq!(err.to_string(), "[line 1] error: operand must be a number");

        assert!(vm.stack.is_empty());
    }

    #[rstest]
    #[case(Value::Nil, 2.0.into())]
    #[case(true.into(), 2.0.into())]
    #[case(8.0.into(), Value::Nil)]
    #[case(8.0.into(), false.into())]
    #[case(true.into(), false.into())]
    fn op_binary_rejects_non_numbers(
        #[values(
            OpCode::OP_GREATER,
            OpCode::OP_LESS,
            OpCode::OP_ADD,
            OpCode::OP_SUBTRACT,
            OpCode::OP_MULTIPLY,
            OpCode::OP_DIVIDE
        )]
        opcode: OpCode,
        #[case] left: Value,
        #[case] right: Value,
    ) {
        let mut chunk = Chunk::new();
        chunk.write_opcode(opcode, 1);

        let mut vm = VM::new(&chunk);
        // keep a value below the operands to ensure that
        // the stack is actually cleared after an error,
        // rather than merely emptied when the operands
        // are popped.
        vm.stack.push(Value::Nil).unwrap();
        vm.stack.push(left).unwrap();
        vm.stack.push(right).unwrap();

        let res = vm.run();
        assert!(matches!(&res, Err(Error::Runtime(_))));

        let err = res.unwrap_err();
        assert_eq!(
            err.to_string(),
            "[line 1] error: both operands must be numbers"
        );
        assert!(vm.stack.is_empty());
    }
}
