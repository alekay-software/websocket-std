use std::cell::RefCell;
use std::net::TcpStream;
use std::io::{BufReader, BufRead, Read, Write, self};
use std::{format};
use crate::result::WebSocketError;
use crate::ws_basic::mask;
use crate::ws_basic::header::{Header, OPCODE, FLAG};
use crate::ws_basic::frame::{DataFrame, Frame};
use crate::core::traits::Serialize;
use super::result::WebSocketResult;
use crate::http::request::{Request, Method};
use std::collections::HashMap;
use core::str;

const DEFAULT_MESSAGE_SIZE: u64 = 1024;

fn generate_key() -> String {
    return String::from("dGhlIHNhbXBsZSBub25jZQ==");
}

pub fn sync_connect<'a>(host: &'a str, port: u16, path: &'a str) -> WebSocketResult<SyncClient<'a>> {
    // Create a tcpstream to the host
    let mut socket = TcpStream::connect(format!("{}:{}", host, port.to_string()))?;

    let key = generate_key();

    let headers = HashMap::from([
        ("Upgrade", "websocket"),
        ("Connection", "Upgrade"),
        ("Sec-WebSocket-Key", key.as_str()),
        ("Sec-WebSocket-Version", "13"),
        ("User-agent", "rust-websocket-std"),
    ]);

    let request = Request::new(Method::GET, path, "HTTP/1.1", Some(headers));

    // Check result of write
    socket.write_all(request.serialize().as_slice())?;

    // Ensure that all data was sent
    socket.flush()?;

    let mut reader = BufReader::new(socket.try_clone()?);

    // Read current current data in the TcpStream
    let mut buffer = String::new();
    let bytes_readed = reader.fill_buf()?.read_to_string(&mut buffer)?;
    
    // Mark the bytes read as consumed so the buffer will not return them in a subsequent read
    reader.consume(bytes_readed);

    // ----------------- Need to confirm that the handshake was accepted ----------------- //
    println!("[HANDSHAKE]: {}", buffer);
    // ----------------------------------------------------------------------------------- //
    
    // Set socket to non-blocking mode
    socket.set_nonblocking(true)?;
    let stream = socket.try_clone()?;
    let client = SyncClient::new(host, port, path, RefCell::new(socket), reader, stream);

    Ok(client)
}


pub struct SyncClient<'a> {
    host: &'a str,
    port: u16,
    path: &'a str,
    message_size: u64,
    writer: RefCell<TcpStream>,
    reader: BufReader<TcpStream>,
    response_cb: Option<fn(String)>,
    stream: TcpStream
}

impl<'a> SyncClient<'a> {
    fn new(host: &'a str, port: u16, path: &'a str, writer: RefCell<TcpStream>, reader: BufReader<TcpStream>, stream: TcpStream) -> Self {
        SyncClient { host, port, path, message_size: DEFAULT_MESSAGE_SIZE, writer, reader, response_cb: None, stream }
    }

    pub fn set_message_size(&mut self, size: u64) {
        self.message_size = size;
    }

    pub fn set_response_cb(&mut self, cb: fn(String)) {
        self.response_cb = Some(cb);
    }

    pub fn send_message(&self, payload: String) -> WebSocketResult<()> {
        // Send single message
        if payload.len() as u64 <= self.message_size {
            let header = Header::new(FLAG::FIN, OPCODE::TEXT, Some(mask::gen_mask()), payload.len() as u64);
            let dataframe = DataFrame::new(header, payload.as_bytes().to_vec());
            self.writer.borrow_mut().write_all(dataframe.serialize().as_slice())?;
    
        // Split into frames and send 
        } else {
            // let frames = payload.len() as u64 / self.frame_size;
            let mut data_sent = 0;
            while data_sent < payload.len() {
                let mut i = data_sent + self.message_size as usize; 
                if i >= payload.len() { i = payload.len() };
                let payload_chunk = payload[data_sent..i].as_bytes();
                let flag = if data_sent + self.message_size as usize >= payload.len() { FLAG::FIN } else { FLAG::NOFLAG };
                let code = if data_sent == 0 { OPCODE::TEXT } else { OPCODE::CONTINUATION };
                let header = Header::new(flag, code, Some(mask::gen_mask()), payload_chunk.len() as u64);
                let frame = DataFrame::new(header, payload_chunk.to_vec());
                self.writer.borrow_mut().write_all(frame.serialize().as_slice())?;
                data_sent += self.message_size as usize;
            }
        }
        
        Ok(())
    }

    // // Try to read a message send from the server
    // pub fn event_loop(&mut self) -> WebSocketResult<()> {
    //     let frame = self.reader.buffer().to_vec();
    //     // This functions try to fill the buffer if is empty
    //     // let frame = self.reader.fill_buf()?.to_vec();

    //     // Clear the buffer
    //     self.reader.consume(frame.len());

    //     if frame.len() <= 0 { return Ok(()) }

    //     // Parse frame
    //     // For now just take the response from the server (assuming is not masked)
    //     // And response is less than 125 characters
    //     let frame = frame;
    //     let response_len = frame[1] & 0b01111111;
    //     let (headers, response) = frame.split_at(2);
    //     let s = str::from_utf8(response);

    //     // self.response_cb(s?.to_string());
    //     let cb = self.response_cb;
    //     if cb.is_some() { cb.unwrap()(s?.to_string()) }

    //     Ok(())
    // }

    pub fn event_loop(&mut self) -> WebSocketResult<()> {
        self.stream.set_nonblocking(true);
        let mut buff = Vec::new();
        match self.stream.read_to_end(&mut buff) {
            // There is data in the socket
            Ok(_) => {
                if self.response_cb.is_some() { 
                    self.response_cb.unwrap()(String::from_utf8(buff).unwrap());
                } 
                return Ok(());
            },

            Err(e) => {
                // wait until network socket is ready, typically implemented
                // via platform-specific APIs such as epoll or IOCP
                // Pass because no messages was receive
                if e.kind() == io::ErrorKind::WouldBlock { return Ok(()); }

                // TODO
                else { return Ok(()); }
            },
        };
    }
}

// TODO: Refactor de code
impl<'a> Drop for SyncClient<'a> {
    fn drop(&mut self) {
        // Send close frame (Connection close normal)
    let mut frame: Vec<u8> = vec![];
    const CLOSE_NORMAL: u16 = 1000;
            
    let mut header1: u8 = 0b10000000;
    let mut header2: u8 = 0b10000000;

    // Ensure that reason is not greater than 125 bytes (payload max length for control frames)
    let reason = String::from("Done");
    header1 |= OPCODE::CLOSE.bits();
    header2 |= (reason.len() + 2) as u8;

    let mask = mask::gen_mask();

    // Add header1, header2 and mask to the frame
    frame.push(header1);
    frame.push(header2);
    frame.extend(mask);

    let mut app_data: Vec<u8> = vec![];
    app_data.extend(CLOSE_NORMAL.to_be_bytes());
    app_data.extend(reason.as_bytes());

    // Mask reaseon
    let mut i = 0;
    for byte in app_data {
        frame.push(byte ^ mask[i]);
        i += 1;
        if i >= 4 { i = 0 }
    }

    println!("{:?}", frame);

    // Send close frame (Wath to do with result or error?)
    self.writer.borrow_mut().write_all(frame.as_slice());

    let mut data = self.reader.fill_buf().unwrap().to_vec();
    self.reader.consume(data.len());

    // Just to test
    // Take directly the bytes from the application data
    let mut status_code_bytes: [u8; 2] = [0; 2];
    status_code_bytes[0] = data[2];
    status_code_bytes[1] = data[3];

    let payload_len: u8 = data[1];

    // Read status code
    let status_code: u16 = u16::from_be_bytes(status_code_bytes);

    println!("DATA: {:?}", data);

    // Delete data until get reason
    data.remove(0);
    data.remove(0);
    data.remove(0);
    data.remove(0);

    // Get Reason (I know that the reason length is 6 (2 (status_code) + 4 (reason)) but it's necessary to read from the header)
    println!("Connection closed (Payload Length): {}", payload_len);
    println!("Connection closed: {}", String::from_utf8(data).unwrap());
    }  
}