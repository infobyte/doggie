#[derive(Debug)]
pub enum BuildError {
    BufferIsFull,
    IndexOutOfBounds,
    BadArguments,
}
