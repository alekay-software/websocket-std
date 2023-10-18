use rand;
use base64;
use sha1_smol::{Sha1, Digest};

// Globally Unique Identifier
const GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub fn gen_key() -> String {
    let key: [u8; 16] = rand::random();
    let key = base64::encode(&key);
    return key;
}

pub fn verify_key(sec_websocket_key: &str, sec_websocket_accept: &str) -> bool {
    let mut key = sec_websocket_key.trim().to_string();
    key.push_str(GUID);
    let mut hasher = sha1_smol::Sha1::new();
    hasher.update(key.as_bytes());
    let accept_key = hasher.digest().to_string();

    accept_key.len() == 20 && sec_websocket_accept == accept_key.as_str()
}