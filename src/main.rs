use rslox::chunk::Chunk;
use rslox::chunk::OpCode::{OP_CONSTANT, OP_RETURN};

fn main() -> color_eyre::Result<()> {
    let mut chunk = Chunk::new();

    let index = chunk.add_constant(1.2);
    chunk.write_opcode(OP_CONSTANT, 0);
    chunk.write_byte(index as u8, 0);
    chunk.write_opcode(OP_RETURN, 0);
    chunk.write_opcode(OP_RETURN, 1);

    // dbg!(&chunk);
    chunk.disasemble("test chunk");

    Ok(())
}
