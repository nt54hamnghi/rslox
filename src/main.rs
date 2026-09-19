use rslox::chunk::Chunk;
use rslox::chunk::OpCode::OP_RETURN;

fn main() -> color_eyre::Result<()> {
    let mut chunk = Chunk::new();
    chunk.write_opcode(OP_RETURN);
    chunk.write_byte(32);
    chunk.disasemble("test chunk");

    Ok(())
}
