use rand;

pub type Mask = [u8; 4];

pub fn gen_mask() -> Mask {
    return rand::random();
}