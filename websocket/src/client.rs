use std::net::{TcpStream, Shutdown};
use std::io::{BufReader, BufRead, Read, Write};
use std::format;
use crate::core::net::read_entire_tcp_package;
use crate::result::WebSocketError;
use crate::ws_basic::mask;
use crate::ws_basic::header::{Header, OPCODE, FLAG};
use crate::ws_basic::frame::{DataFrame, ControlFrame, Frame, parse, FrameKind};
use crate::core::traits::Serialize;
use super::result::WebSocketResult;
use crate::http::request::{Request, Method};
use std::collections::HashMap;
use core::str;

const DEFAULT_MESSAGE_SIZE: u64 = 1024;

// TODO: Generate a random string key and store into the client
fn generate_key() -> String {
    return String::from("dGhlIHNhbXBsZSBub25jZQ==");
}

// TODO: Confirm that the handshake is accepted
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
    let client = SyncClient::new(host, port, path, &stream);

    Ok(client)
}

// [] TODO: Implement framable trait (trait to split the data into frames)
// [] TODO: Create a trait to send and receive data from the websocket
// [x] TODO: Queues for messages to send
// [] TODO: Queues for messages to receive
// [x] TODO: Event loop must send messages from the queues
// [] TODO: Event loop must receive messages from the queues
// [] TODO: Decide if write or read messages
pub struct SyncClient<'a> {
    host: &'a str,
    port: u16,
    path: &'a str,
    message_size: u64,
    response_cb: Option<fn(&str)>,
    stream: TcpStream,
    reader: BufReader<TcpStream>,
    // Store frames that the client wants to send to the websocket
    send_queue: Vec<Box<dyn Frame>>
}

impl<'a> SyncClient<'a> {
    fn new(host: &'a str, port: u16, path: &'a str, stream: &TcpStream) -> Self {
        SyncClient { host, port, path, message_size: DEFAULT_MESSAGE_SIZE, response_cb: None, stream: stream.try_clone().unwrap(), reader: BufReader::new(stream.try_clone().unwrap()), send_queue: Vec::new() }
    }

    // TODO: The message size does not take into account
    pub fn set_message_size(&mut self, size: u64) {
        self.message_size = size;
    }

    // TODO: This function is only for text messages, pass to the callback information about the type of the frame
    pub fn set_response_cb(&mut self, cb: fn(&str)) {
        self.response_cb = Some(cb);
    }

    pub fn send_message(&mut self, payload: &'a str) -> WebSocketResult<()> {
        let mut frames: Vec<Box<dyn Frame>> = Vec::new();

        // Send single message
        if payload.len() as u64 <= self.message_size {
            let header = Header::new(FLAG::FIN, OPCODE::TEXT, Some(mask::gen_mask()), payload.len() as u64);
            let dataframe = DataFrame::new(header, payload.as_bytes().to_vec());
            frames.push(Box::new(dataframe));
    
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
                frames.push(Box::new(frame));
                data_sent += self.message_size as usize;
            }
        }
        
        self.send_queue.extend(frames);

        Ok(())
    }

    // TODO: I'm assuming a lot of wrong things
    // TODO: What's happend if a close frame es received from the server?
    pub fn event_loop(&mut self) -> WebSocketResult<()> {

        // handle one frame from the send_queue
        let frame = self.send_queue.pop();

        if frame.is_some() {
            let frame = frame.unwrap();
            let serialized_frame = frame.serialize();
            self.stream.write_all(serialized_frame.as_slice())?;
        }

        // Read new frame from the server
        let data = read_entire_tcp_package(&mut self.reader)?;
        
        // If the frame is not the last, keep in memory to continue append following data
        
        if data.len() > 0 && self.response_cb.is_some() {
            // Get data from the frame
            // If is last frame send all the message
            // Right now assume that the data is an unique frame
            // If the last message was not readed completelly, add the other part
            
            let frames = parse(data.as_slice())?;
            
            // TODO: More than osdne frame could be receive in the same tcp package     
            // TODO: Is last frame? or need to store until the rest of the parts arrive?

            for frame in frames {
                match frame.kind()  {
                    FrameKind::Data => { self.response_cb.unwrap()(str::from_utf8(frame.get_data())?); },
                    FrameKind::Control => { self.handle_control_frame(frame.as_any().downcast_ref::<ControlFrame>().unwrap()); },
                    FrameKind::NotDefine => return Err(WebSocketError::ProtocolError("OPCODE not supported"))
                };
            }
            
        }
        Ok(())
    }

    fn handle_control_frame(&mut self, frame: &ControlFrame) -> WebSocketResult<()> {
        match frame.get_header().get_opcode() {
            OPCODE::PING=> { 
                // Create a PONG frame. Set masked App data if the PING frame contains any App data
                // println!("[CLIENT]: PING received data -> {}", String::from_utf8(frame.get_data().to_vec()).unwrap());
                let data = frame.get_data();
                let mask = if data.len() > 0 { Some(mask::gen_mask()) } else { None };
                let header = Header::new(FLAG::FIN, OPCODE::PONG, mask, data.len() as u64);
                let pong_frame = ControlFrame::new(header, None, data.to_vec());
                println!("[CLIENT]: Sending pong");
                self.stream.write_all(pong_frame.serialize().as_slice())?;

            },
            OPCODE::PONG => { todo!("Not implemented handle PONG") },
            OPCODE::CLOSE => { todo!("Not implemented handle CLOSE frame")},
            _ => return Err(WebSocketError::ProtocolError("Invalid OPCODE for Control Frame"))
        }

        Ok(())
    }
}

// TODO: Refactor de code
impl<'a> Drop for SyncClient<'a> {
    fn drop(&mut self) {
    
    // TODO: Send all messages in the queue
    // TODO: Read messages and send responses until close frame is received in response of our close frame

    let msg = "Done";
    let mask = Some(mask::gen_mask());
    let header = Header::new(FLAG::FIN, OPCODE::CLOSE, mask, msg.len() as u64 + 2);
    let status_code: u16 = 1000;
    let control_frame = ControlFrame::new(header, Some(status_code), msg.as_bytes().to_vec());

    self.stream.set_nonblocking(false);

    self.stream.write_all(control_frame.serialize().as_slice());

    // At this point the client could receive messages from the server
    // Send response for all messages and wait until close is received
    // TODO: Use read function from the client to read from the socket
    let mut data = self.reader.fill_buf().unwrap().to_vec();
    self.reader.consume(data.len());

    // Try parse data into a frame
    // TODO: Wait until close frame is received
    let response = parse(data.as_slice());

    match response {
        Ok(frames) => {
            // let d = str::from_utf8(frame.get_data());
            let data = frames[0].get_data();
            let msg =  str::from_utf8(&data[2..data.len()]).unwrap();
            println!("[CLIENT CLOSE]: Response from server -> {}", msg);
        },
        Err(_) => { }
    }

    self.stream.shutdown(Shutdown::Both);
    // // Set socket to block mode
    // self.stream.set_nonblocking(false);

    // // Send close frame (Connection close normal)
    // let mut frame: Vec<u8> = vec![];
    // const CLOSE_NORMAL: u16 = 1000;
            
    // let mut header1: u8 = 0b10000000;
    // let mut header2: u8 = 0b10000000;

    // // Ensure that reason is not greater than 125 bytes (payload max length for control frames)
    // let reason = String::from("Done");
    // header1 |= OPCODE::CLOSE.bits();
    // header2 |= (reason.len() + 2) as u8;

    // let mask = mask::gen_mask();

    // // Add header1, header2 and mask to the frame
    // frame.push(header1);
    // frame.push(header2);
    // frame.extend(mask);

    // let mut app_data: Vec<u8> = vec![];
    // app_data.extend(CLOSE_NORMAL.to_be_bytes());
    // app_data.extend(reason.as_bytes());

    // // Mask reaseon
    // let mut i = 0;
    // for byte in app_data {
    //     frame.push(byte ^ mask[i]);
    //     i += 1;
    //     if i >= 4 { i = 0 }
    // }

    // println!("{:?}", frame);

    // // Send close frame (Wath to do with result or error?)
    // self.stream.write_all(frame.as_slice());


    // // At this point the client could receive messages from the server
    // // Send response for all messages and wait until close is received
    // let mut data = self.reader.fill_buf().unwrap().to_vec();
    // self.reader.consume(data.len());

    // // Just to test
    // // Take directly the bytes from the application data
    // let mut status_code_bytes: [u8; 2] = [0; 2];
    // status_code_bytes[0] = data[2];
    // status_code_bytes[1] = data[3];

    // let payload_len: u8 = data[1];

    // // Read status code
    // let status_code: u16 = u16::from_be_bytes(status_code_bytes);

    // // Read reason
    // let (_, reason) = data.split_at(4);

    // println!("DATA: {:?}", data);
    

    // // Get Reason (I know that the reason length is 6 (2 (status_code) + 4 (reason)) but it's necessary to read from the header)
    // println!("Connection closed (Payload Length): {}", payload_len);
    // println!("Connection closed: {}", String::from_utf8(reason.to_vec()).unwrap());
    }  
}