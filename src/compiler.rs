use crate::chunk::Chunk;
use crate::errors::Error;
use crate::scanner::Scanner;

pub fn compile(source: &str, chunk: &mut Chunk) -> Result<(), Error> {
    let scanner = Scanner::new(source);
    Ok(())
}
