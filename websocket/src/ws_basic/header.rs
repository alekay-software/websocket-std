use super::mask::Mask;
use super::super::core::traits::Serialize;

use bitflags::bitflags;

bitflags! {
    // Type of the frame
    pub struct OPCODE: u8 {
        const CONTINUATION = 0x0;
        const TEXT = 0x1;
        const BINARY = 0x2;
        const CLOSE = 0x8;
        const PING = 0x9;
        const PONG = 0xA;
    }
}

bitflags! {
    pub struct FLAG: u8 {
        // Mark this frame the last of the secuence
        const FIN = 0x80;   //  10000000
        // Mark as not last
        const NOFLAG = 0x00;   //  10000000
        // Reserved bit (1)
        const RSV1 = 0x40;  //  01000000
        // Reserved bit (2)
        const RSV2 = 0x20;  //  00100000
        // Reserved bit (3)
        const RSV3 = 0x10;  //  00010000
    }
}

// pub struct Header {
//     code: OPCODE,
//     mask_key: Option<Mask>,
//     payload_len: u64
// }

// impl Header {
//     pub fn new(code: OPCODE, mask_key: Option<Mask>, payload_len: u64) -> Self {
//         Header { code, mask_key: mask_key, payload_len }
//     }

//     pub fn get_mask(&self) -> Option<Mask> {
//         return self.mask_key;
//     }

//     pub fn is_control_header(&self) -> bool {
//         self.code.bits() >= 8
//     }
// }

// impl Serialize for Header {
//     fn serialize(&self) -> Vec<u8> {
//         let mut buffer: Vec<u8> = vec![];

//         let mut header1: u8 = 0b10000000;

//         // OR with OPCODE to get the first part of the header
//         header1 |= self.code.bits();

//         buffer.push(header1);

//         // Mask bit + Payload len
//         let mut header2 = if self.mask_key.is_some() { 0b10000000 } else { 0b00000000 };

//         if self.payload_len < 125 {
//             header2 |= self.payload_len as u8;
//             buffer.push(header2);
//         } else if self.payload_len <=  65535 { // 65535 = 2ˆ16 - 1 (max unsigned integer that can be represented with 16 bits)
//             header2 |= 126;
//             buffer.push(header2);
//             buffer.extend((self.payload_len as u16).to_be_bytes());
//         } else {
//             header2 |= 127; // Payload len represented by a 64 bits number
//             buffer.push(header2);
//             buffer.extend(self.payload_len.to_be_bytes());
//         }

//         if self.mask_key.is_some() {
//             // Add mask
//             let mask = self.mask_key.unwrap();
//             buffer.extend(mask);
//         }

//         return buffer;
//     }
// }

pub struct Header {
    // FIN RSV1 RSV2 RSV3
    flag: FLAG,
    // Type of the frame (TEXT, CLOSE, PING, PONG...)
    code: OPCODE,
    // If None then the frame dont need to be masked, otherwise, mask with the value
    mask_key: Option<Mask>,
    // Len of the payload
    payload_len: u64
}

impl Header {
    pub fn new(flag: FLAG, code: OPCODE, mask_key: Option<Mask>, payload_len: u64) -> Self {
        Header { flag, code, mask_key, payload_len }
    }

    pub fn get_mask(&self) -> Option<Mask> {
        self.mask_key
    } 
}

impl Serialize for Header {
    fn serialize(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = vec![];

        let mut header1: u8 = self.flag.bits();

        // OR with OPCODE to get the first part of the header
        header1 |= self.code.bits();

        buffer.push(header1);

        // Mask bit + Payload len
        let mut header2 = if self.mask_key.is_some() { 0b10000000 } else { 0b00000000 };

        if self.payload_len < 125 {
            header2 |= self.payload_len as u8;
            buffer.push(header2);
        } else if self.payload_len <=  65535 { // 65535 = 2ˆ16 - 1 (max unsigned integer that can be represented with 16 bits)
            header2 |= 126;
            buffer.push(header2);
            buffer.extend((self.payload_len as u16).to_be_bytes());
        } else {
            header2 |= 127; // Payload len represented by a 64 bits number
            buffer.push(header2);
            buffer.extend(self.payload_len.to_be_bytes());
        }

        if self.mask_key.is_some() {
            // Add mask
            let mask = self.mask_key.unwrap();
            buffer.extend(mask);
        }

        return buffer;
    }
}