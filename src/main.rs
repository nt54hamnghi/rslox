use rslox::chunk::{Chunk, OpCode};
use rslox::vm::VM;

fn main() -> color_eyre::Result<()> {
    let mut chunk = Chunk::new();

    chunk.write_constant(1.2, 1);
    chunk.write_constant(3.4, 1);
    chunk.write_opcode(OpCode::OP_RETURN, 2);
    // chunk.disasemble("test");

    VM::interpret(&chunk);

    Ok(())
}
