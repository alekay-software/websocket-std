use rand;
use base64;

pub fn gen_ws_key() -> String {
    let key: [u8; 16] = rand::random();
    let key = base64::encode(&key);
    return key;
}