use rslox::chunk::{Chunk, OpCode};
use rslox::vm::VM;

fn main() -> color_eyre::Result<()> {
    let mut chunk = Chunk::new();

    chunk.write_constant(4.0, 1);
    chunk.write_constant(3.0, 1);
    chunk.write_constant(2.0, 1);
    chunk.write_opcode(OpCode::OP_NEGATE, 1);
    chunk.write_opcode(OpCode::OP_MULTIPLY, 1);
    chunk.write_opcode(OpCode::OP_SUBTRACT, 1);
    chunk.write_opcode(OpCode::OP_RETURN, 2);
    #[cfg(feature = "trace")]
    {
        chunk.disasemble("test");
    }

    VM::interpret(&chunk);

    Ok(())
}
