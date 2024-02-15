use std::fmt;

pub trait Serialize {
    fn serialize(&self) -> Vec<u8>;
}

#[derive(Debug)]
pub struct ParseError;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error parsing")
    }
}

pub trait Parse {
    fn parse(bytes: &[u8]) -> Result<Self, ParseError> where Self: Sized;
}