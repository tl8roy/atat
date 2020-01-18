
use ufmt::derive::uDebug;

#[derive(uDebug, Debug, Clone, Copy)]
pub enum ATATError {
    ParseError,
    CreateError,
    BufferError,
    UnknownCommandError,
    SerialError,
}