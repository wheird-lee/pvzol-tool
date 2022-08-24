use std::error::Error;

pub type Id = usize;

pub type Grade = u32;

pub mod sys;
pub mod user;

pub(super) type Result<T> = std::result::Result<T, ErrorKind>;

pub enum ErrorKind {
    DataNotInitialized,
    Other(Box<dyn Error>),
}

impl ErrorKind {
    pub(crate) fn other_str(string: &str) -> Self {
        ErrorKind::Other(string.into())
    }
}

pub struct GameUser {
    
}
