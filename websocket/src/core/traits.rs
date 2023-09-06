pub trait Serialize {
    fn serialize(&self) -> Vec<u8>;
}

pub trait Parse {
    fn parse(bytes: &[u8]) -> Self;
}