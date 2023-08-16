use std::any::Any;
use crate::result::{WebSocketResult, WebSocketError};
use super::{header::{Header, FLAG, OPCODE}, mask::Mask};
use super::super::core::traits::Serialize;
use super::super::core::binary::{bytes_to_u16, bytes_to_u64};
use std::io::{Read};
use super::super::core::net::read_into_buffer;

pub enum FrameKind {
    Data,
    Control,
    NotDefine
}

pub trait Frame {
    // Return the data containing in the frame
    fn get_data(&self) -> &[u8];
    // Return the header struct of the frame
    fn get_header(&self) -> &Header;
    // Downcast to concrete type
    fn as_any(&self) -> &dyn Any;

    // Return the type of the frame (Dataframe or controlframe)
    fn kind(&self) -> FrameKind {
        let opcode = self.get_header().get_opcode();
        if opcode == OPCODE::CLOSE || opcode == OPCODE::PING || opcode == OPCODE::PONG  {
            return FrameKind::Control;
        } else if opcode == OPCODE::BINARY  || opcode == OPCODE::TEXT  || opcode == OPCODE::CONTINUATION  {
            return FrameKind::Data;
        } else {
            return FrameKind::NotDefine;
        }
    }

    // Return the byte representation of the frame, useful to send through a socket
    fn serialize(&self) -> Vec<u8> {
        let mut serialized_data = vec![];
        
        serialized_data.extend(self.get_header().serialize());

        match self.get_header().get_mask() {
            // Apply mask to data
            Some(mask) => {
                let mut i = 0;
                for &byte in self.get_data() {
                    serialized_data.push(byte ^ mask[i]);
                    i += 1;
                    if i >= mask.len() { i = 0 };
                }
            },
            // Just insert App data without mask
            None => serialized_data.extend(self.get_data())
        }

        return serialized_data; 
    }
}

// Dataframe struct
pub struct DataFrame {
    header: Header,
    data: Vec<u8>
}

impl DataFrame {
    pub fn new(header: Header, data: Vec<u8>) -> Self {
        DataFrame { header, data }
    }
}

impl Frame for DataFrame {
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn get_data(&self) -> &[u8] {
        self.data.as_slice()
    }

    fn get_header(&self) -> &Header {
        &self.header
    }
}

// ControlFrame struct
pub struct ControlFrame {
    header: Header,
    data: Vec<u8>,
}

impl ControlFrame {
    pub fn new(header: Header, status_code: Option<u16>, data: Vec<u8>) -> Self {
        // Data len should be less than 123 (2-bytes of status code + data <= 125) Where I shold veritfy this condition?
        let mut merge_data = vec![];
        if status_code.is_some() {
            merge_data.extend(status_code.unwrap().to_be_bytes());
        }
        merge_data.extend(data);
        ControlFrame { header, data: merge_data }
    }
}

impl Frame for ControlFrame {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_data(&self) -> &[u8] {
        self.data.as_slice()
    }

    fn get_header(&self) -> &Header {
        &self.header
    }
}

// Parse bytes into frames
pub fn parse(bytes: &[u8]) -> WebSocketResult<Vec<Box<dyn Frame>>> {
    // Check if all index using to get the content from the frame are not out of the array
    let mut frames: Vec<Box<dyn Frame>> = Vec::new();
    let mut frame = bytes;
    
    loop {
        // Parse bytes into a Frame
        if frame.len() < 2 {
            let mut msg = String::from("Error parsing a frame, frame length must be >= 2, got: ");
            msg.push_str(frame.len().to_string().as_str());
            return Err(WebSocketError::Custom(msg));
        }

        // Flag
        let flag = FLAG::from_bits(frame[0] & 0b11110000);
        if flag.is_none() { 
            let mut msg = String::from("Invalid flag: ");
            msg.push_str(frame[0].to_string().as_str());
            return Err(WebSocketError::Custom(msg));
        }
        
        // code
        let code = OPCODE::from_bits(frame[0] & 0b000011111);
        if  code.is_none() { 
            let mut msg = String::from("Invalid opcode: ");
            msg.push_str(frame[1].to_string().as_str());
            return Err(WebSocketError::Custom(msg));
        }

        let is_masked = (0b10000000 & frame[1]) == 1;

        // Payload length
        let mut payload_len: u64 = 0b01111111 as u64 & frame[1] as u64;
        let mut i = 2; // Index to know the start point of the mask if exists

        if payload_len == 126 {
            i = 4;
            payload_len = bytes_to_u16(&frame[2..4]).unwrap() as u64;
        } else if payload_len == 127 {
            i = 10;
            payload_len = bytes_to_u64(&frame[2..10]).unwrap();
        }

        // Frame not received completelly due to buffers from the OS
        if payload_len + i as u64 > frame.len() as u64 { return Err(WebSocketError::Custom(String::from("Frame is not completelly readed"))); }

        // Mask Key
        let mut mask: Option<Mask> = None;
        if is_masked {
            let mut buf: [u8; 4] = [0,0,0,0];
            for j in 0..4 {
                buf[j] = frame[i+j];
            }
            mask = Some(buf);
            i += 4;
        }

        let code = code.unwrap();
        let code_bits = code.bits();
        let header = Header::new(flag.unwrap(), code, mask, payload_len); 
        

        // Dataframe
        if code_bits == OPCODE::TEXT.bits() || code_bits == OPCODE::BINARY.bits() || code_bits == OPCODE::CONTINUATION.bits() {
            let data = &frame[i..payload_len as usize +i];
            frames.push(Box::new(DataFrame::new(header, data.to_vec())));

        // ControlFrame
        } else {
            let status_code = bytes_to_u16(&frame[i..i+2]).unwrap();
            let data = &frame[i+2..payload_len as usize + 2];
            frames.push(Box::new(ControlFrame::new(header, Some(status_code), data.to_vec())));
        }

        frame = &frame[i + payload_len as usize .. frame.len()];

        if frame.len() == 0 { break } // Al frames readed

    }

    return Ok(frames);

}


// try to convert the bytes into a frame
// the offset means the bytes that are left over bytes[offset, end]
pub fn bytes_to_frame(bytes: &[u8]) -> WebSocketResult<Option<(Box<dyn Frame>, usize)>>{
    if bytes.len() < 2 {
        let mut msg = String::from("Error parsing a frame, frame length must be >= 2, got: ");
        msg.push_str(bytes.len().to_string().as_str());
        return Ok(None);
        // This is not an error, bytes could arrive later
    }

    // Flag
    let flag = FLAG::from_bits(bytes[0] & 0b11110000);
    if flag.is_none() { 
        let mut msg = String::from("Invalid flag: ");
        msg.push_str(bytes[0].to_string().as_str());
        return Err(WebSocketError::Custom(msg));
    }
    
    // code
    let code = OPCODE::from_bits(bytes[0] & 0b000011111);
    if  code.is_none() { 
        let mut msg = String::from("Invalid opcode: ");
        msg.push_str(bytes[1].to_string().as_str());
        return Err(WebSocketError::Custom(msg));
    }

    let is_masked = (0b10000000 & bytes[1]) == 1;

    // Payload length
    let mut payload_len: u64 = 0b01111111 as u64 & bytes[1] as u64;
    let mut i = 2; // Index to know the start point of the mask if exists

    if payload_len == 126 {
        i = 4;
        payload_len = bytes_to_u16(&bytes[2..4]).unwrap() as u64;
    } else if payload_len == 127 {
        i = 10;
        payload_len = bytes_to_u64(&bytes[2..10]).unwrap();
    }

    // bytes not received completelly due to buffers from the OS
    if payload_len + i as u64 > bytes.len() as u64 { return Ok(None) }
    // if payload_len + i as u64 > bytes.len() as u64 { return Err(WebSocketError::Custom(String::from("Frame is not completelly readed"))); }

    // Mask Key
    let mut mask: Option<Mask> = None;
    if is_masked {
        let mut buf: [u8; 4] = [0,0,0,0];
        for j in 0..4 {
            buf[j] = bytes[i+j];
        }
        mask = Some(buf);
        i += 4;
    }

    let code = code.unwrap();
    let code_bits = code.bits();
    let header = Header::new(flag.unwrap(), code, mask, payload_len); 

    let offset = i + payload_len as usize;

    // Dataframe
    if code_bits == OPCODE::TEXT.bits() || code_bits == OPCODE::BINARY.bits() || code_bits == OPCODE::CONTINUATION.bits() {
        let data = &bytes[i..payload_len as usize +i];
        return Ok(Some((Box::new(DataFrame::new(header, data.to_vec())), offset)));

    // ControlFrame
    } else {
        let status_code = bytes_to_u16(&bytes[i..i+2]).unwrap();
        let data = &bytes[i+2..payload_len as usize + 2];
        return Ok(Some((Box::new(ControlFrame::new(header, Some(status_code), data.to_vec())), offset)));
    }

    // if bytes.len() == 0 { break } // Al frames readed
}

/*
Reads an entire frame from the Reader.
When using TCPStreams, to get the full message multiples calls to read method
has to be done because the reader has a buffer capacity. If the amount of data is greatest than the capacity of 
the buffer, then you should read multiples times in order to read the entire data.
 */
// pub fn read_entire_frames(reader: &mut dyn Read, storage: &mut Vec<u8>) -> Result<Vec<u8>, WebSocketError> {
//     // TODO: Send the buffer into the function parameters, to decide how much data can the system read at one time
//     let mut buf: [u8; 1024] = [0; 1024];
//     let mut frame;


//     loop {
//         match reader.read(&mut buf) {
//             // Reached end of file (error in the connection)
//             Ok(data) => {
//                 if data <= 0 {
//                     let e = Error::new(io::ErrorKind::ConnectionReset, "Connection reset by peer");
//                     return Err(e);
//                 } else {
//                     storage.extend_from_slice(&buf.as_slice()[0..data]);
//                     let result = bytes_to_frame(storage.as_slice());
//                     match result {
//                         Ok((f, offset)) => {
//                             frame = f;
//                             storage.remove(index)
//                         },
//                         Err(e) => { return Err(e); }
//                     }
//                 }

//             },
//             Err(e) => {
//                 if e.kind() == io::ErrorKind::WouldBlock { break }
//                 else { return Err(WebSocketError::IOError(e)) }
//             }
//         }
//     }

//     return Ok(frame);
// }/

// TODO: Pass buffer to this function
pub fn read_frame(reader: &mut dyn Read, storage: &mut Vec<u8>) -> WebSocketResult<Option<Box<dyn Frame>>> {

    // Try to get a frame from previous bytes readed from the socket
    if storage.len() > 0 { 
        let res = bytes_to_frame(storage.as_slice())?;
    
        if res.is_some() { 
            let (frame, offset) = res.unwrap();
            storage.drain(0..offset);
            return Ok(Some(frame)); 
        }
    }

    // Read from socket
    // TODO: Pass this buffer from the user to control the perfomance
    let mut buf: [u8; 1024] = [0; 1024];

    // use net function to put data into the buffer and try to create a socket with the data in the storage and the data readed
    // If a frame is created, the rest of the data readed from the socket must be save into the storage vector
    let res = read_into_buffer(reader, &mut buf);
    if res.is_err() { return Err(res.unwrap_err()); }

    let amount = res.unwrap();
    if amount <= 0 { return Ok(None); }

    storage.extend_from_slice(&buf[0..amount]);
    let res = bytes_to_frame(storage.as_slice())?;

    if res.is_some() { 
        let (frame, offset) = res.unwrap();
        storage.drain(0..offset);
        return Ok(Some(frame)); 
    }

    return Ok(None);

}