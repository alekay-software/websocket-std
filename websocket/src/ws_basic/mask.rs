use getrandom as rand;

pub type Mask = [u8; 4];

pub fn gen_mask() -> Mask {
    let mut buf: Mask = [0u8; 4];
    let _ = rand::getrandom(&mut buf); // Ignore error
    return buf
}