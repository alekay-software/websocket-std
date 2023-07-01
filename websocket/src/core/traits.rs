pub trait Serialize {
    fn serialize(&self) -> Vec<u8>;
}

pub trait Parse {
    fn parse(bytes: &Vec<u8>) -> Self;
}