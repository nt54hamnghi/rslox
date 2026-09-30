use rslox::chunk::{Chunk, OpCode};

fn main() -> color_eyre::Result<()> {
    let mut chunk = Chunk::new();

    chunk.write_constant(1.2, 0);
    chunk.write_constant(3.4, 0);

    // Fill the constant table so the next value needs a 24-bit index.
    for _ in 2..256 {
        chunk.add_constant(0.0);
    }
    chunk.write_constant(5.6, 1);
    chunk.write_opcode(OpCode::OP_RETURN, 2);

    // dbg!(&chunk);
    chunk.disasemble("test chunk");

    Ok(())
}
