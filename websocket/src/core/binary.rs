use std::convert::TryInto;
use core::array::TryFromSliceError;

/// Return a unsigned 64 bits number from the given bytes asuming big endian representation.
pub fn bytes_to_u64(bytes: &[u8]) -> Result<u64, TryFromSliceError> {
    let res: Result<[u8; 8], _> = bytes.try_into();
    if res.is_err() { return Err(res.err().unwrap()); }
    let buf = res.unwrap();
    return Ok(u64::from_be_bytes(buf));
}

/// Return a unsigned 16 bits number from the given bytes asuming big endian representation.
pub fn bytes_to_u16(bytes: &[u8]) -> Result<u16, TryFromSliceError> {
    let res: Result<[u8; 2], _> = bytes.try_into();
    if res.is_err() { return Err(res.err().unwrap()); }
    let buf = res.unwrap();
    return Ok(u16::from_be_bytes(buf));
}