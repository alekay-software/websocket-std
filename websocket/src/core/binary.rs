// Get the u64 number from the bytes asumming big endian representation
pub fn bytes_to_u64(bytes: &[u8]) -> u64 {
    let mut buf: [u8; 8] = [0,0,0,0,0,0,0,0];
    let len = bytes.len();
    
    let mut i: usize = 0;
    while i < len && i < buf.len() {
        buf[i] = bytes[i];
        i += 1;
    }
    
    return u64::from_be_bytes(buf);
}

// Get the u64 number from the bytes asumming big endian representation
pub fn bytes_to_u16(bytes: &[u8]) -> u16 {
    let mut buf: [u8; 2] = [0,0];
    let len = bytes.len();
    
    let mut i: usize = 0;
    while i < len && i < buf.len() {
        buf[i] = bytes[i];
        i += 1;
    }

    return u16::from_be_bytes(buf);
}