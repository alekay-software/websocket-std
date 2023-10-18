use rand;
use base64;
use sha1_smol::Sha1;

// Globally Unique Identifier
const GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub fn gen_key() -> String {
    let key: [u8; 16] = rand::random();
    return base64::encode(&key);
}

pub fn verify_key(sec_websocket_key: &str, sec_websocket_accept: &str) -> bool {
    let mut accept_key = String::with_capacity(sec_websocket_key.len() + GUID.len());
    accept_key.push_str(sec_websocket_key);
    accept_key.push_str(GUID);
    let mut hasher = Sha1::new();
    hasher.update(accept_key.as_bytes());
    let accept_key = hasher.digest().bytes();
    let accept_key = base64::encode(&accept_key);
    return accept_key.as_str() == sec_websocket_accept;
}