use rslox::chunk::{Chunk, OpCode};

fn main() -> color_eyre::Result<()> {
    let mut chunk = Chunk::new();

    chunk.write_constant(1.2, 0);
    chunk.write_constant(3.4, 0);
    chunk.write_opcode(OpCode::OP_RETURN, 1);

    // dbg!(&chunk);
    chunk.disasemble("test chunk");

    Ok(())
}
