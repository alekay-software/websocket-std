use std::net::{TcpStream, Shutdown};
use std::io::{BufReader, BufRead, Read, Write};
use std::format;
use crate::result::WebSocketError;
use crate::ws_basic::mask;
use crate::ws_basic::header::{Header, OPCODE, FLAG};
use crate::ws_basic::frame::{DataFrame, ControlFrame, Frame, FrameKind, read_frame};
use crate::core::traits::Serialize;
use super::result::WebSocketResult;
use crate::http::request::{Request, Method};
use std::collections::{HashMap, VecDeque};
use core::str;
use std::time::Instant;

const DEFAULT_MESSAGE_SIZE: u64 = 1024;
const DEFAULT_TIMEOUT_SECS: u64 = 30;

#[derive(PartialEq)]
enum EventType {
    SEND,
    RECEIVED
}

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

    let mut reader = BufReader::new(&socket);

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
    let client = SyncClient::new(host, port, path, socket);

    Ok(client)
}

// [] TODO: Implement framable trait (trait to split the data into frames)
// [] TODO: Create a trait to send and receive data from the websocket
// [x] TODO: Queues for messages to send
// [] TODO: Queues for messages to receive
// [x] TODO: Event loop must send messages from the queues
// [] TODO: Event loop must receive messages from the queues
// [] TODO: Decide if write or read messages
// [] TODO: Send the size of the buffer to read data from the stream, so the client will decide the perfomance base on the memory available or the size of the messages that the system is going to receive
pub struct SyncClient<'a> {
    host: &'a str,
    port: u16,
    path: &'a str,
    connection_closed: bool,
    message_size: u64,
    response_cb: Option<fn(String)>,
    stream: TcpStream,
    event_queue: VecDeque<(EventType, Box<dyn Frame>)>,     // Events that the event loop will execute
    recv_storage: Vec<u8>,                                  // Storage to keep the bytes received from the socket
    send_queue: VecDeque<Box<dyn Frame>>,                    // Store frames to send                                
    recv_data: Vec<u8>                                       // Store the data received from the Frames until the data is completelly received
}

impl<'a> SyncClient<'a> {
    fn new(host: &'a str, port: u16, path: &'a str, stream: TcpStream) -> Self {
        SyncClient { host, port, path, connection_closed: false, message_size: DEFAULT_MESSAGE_SIZE, response_cb: None, stream, event_queue: VecDeque::new(), recv_storage: Vec::new(), send_queue: VecDeque::new(), recv_data: Vec::new() }
    }

    // TODO: The message size does not take into account
    pub fn set_message_size(&mut self, size: u64) {
        self.message_size = size;
    }

    // TODO: This function is only for text messages, pass to the callback information about the type of the frame
    pub fn set_response_cb(&mut self, cb: fn(String)) {
        self.response_cb = Some(cb);
    }

    // TODO: Create just one frame to send, if need to create more than one, store the rest of the bytes into a vector
    pub fn send_message(&mut self, payload: String) -> WebSocketResult<()> {
        // Send single message
        if payload.len() as u64 <= self.message_size {
            let header = Header::new(FLAG::FIN, OPCODE::TEXT, Some(mask::gen_mask()), payload.len() as u64);
            let dataframe = DataFrame::new(header, payload.as_bytes().to_vec());
            self.event_queue.push_back((EventType::SEND, Box::new(dataframe)));
    
        // Split into frames and send 
        } else {
            let mut data_sent = 0;
            while data_sent < payload.len() {
                let mut i = data_sent + self.message_size as usize; 
                if i >= payload.len() { i = payload.len() };
                let payload_chunk = payload[data_sent..i].as_bytes();
                let flag = if data_sent + self.message_size as usize >= payload.len() { FLAG::FIN } else { FLAG::NOFLAG };
                let code = if data_sent == 0 { OPCODE::TEXT } else { OPCODE::CONTINUATION };
                let header = Header::new(flag, code, Some(mask::gen_mask()), payload_chunk.len() as u64);
                let frame = DataFrame::new(header, payload_chunk.to_vec());

                // Put the first frame of the split into the event_queue
                if data_sent <= 0 {
                    self.event_queue.push_back((EventType::SEND, Box::new(frame)))
                // The rest of the frames goes to the send_queue
                } else {
                    self.send_queue.push_back(Box::new(frame));
                }
                data_sent += self.message_size as usize;
            }
        }

        Ok(())

    }

    // TODO: I'm assuming a lot of wrong things
    // TODO: What's happend if a close frame es received from the server?
    pub fn event_loop(&mut self) -> WebSocketResult<()> {

        // Try to read Frames from the socket
        let frame = read_frame(&mut self.stream, &mut self.recv_storage)?;
        if frame.is_some() { self.event_queue.push_back((EventType::RECEIVED, frame.unwrap())); }

        // Insert more frames from the send queue
        let frame = self.send_queue.pop_front();
        if frame.is_some() { self.event_queue.push_back((EventType::SEND, frame.unwrap())); }

        // Take one frame from the event loop queue
        let res = self.event_queue.pop_front();
        if res.is_none() { return Ok(()) }  // No events to handle

        // Frame to handle
        let (event_type, frame) = res.unwrap();

        if event_type == EventType::RECEIVED {
            match frame.kind()  {
                FrameKind::Data => { 
                    if frame.get_header().get_flag() != FLAG::FIN {
                        self.recv_data.extend_from_slice(frame.get_data());
                    }

                    if self.response_cb.is_some() {
                        let callback = self.response_cb.unwrap();

                        let res = String::from_utf8(frame.get_data().to_vec());
                        if res.is_err() { return Err(WebSocketError::Utf8Error(res.err().unwrap().utf8_error())); }
                        
                        let msg = res.unwrap();

                        // Message received in a single frame
                        if self.recv_data.is_empty() {
                            callback(msg);

                        // Message from a multiples frames     
                        } else {
                            let previous_data = self.recv_data.clone();
                            let res = String::from_utf8(previous_data);
                            if res.is_err() { return Err(WebSocketError::Utf8Error(res.err().unwrap().utf8_error())); }
                            
                            let mut completed_msg = res.unwrap();
                            completed_msg.push_str(msg.as_str());

                            // Send the message to the callback function
                            callback(completed_msg);

                            // Remove from memory
                            // TODO: Test if is better to just clear the vector and maintain the capacity allocated
                            drop(&self.recv_data);
                            self.recv_data = Vec::new();
                        }
                    }
                },
                FrameKind::Control => { return self.handle_control_frame(frame.as_any().downcast_ref::<ControlFrame>().unwrap()); },
                FrameKind::NotDefine => return Err(WebSocketError::ProtocolError("OPCODE not supported"))
            };
        // EventType == SEND    
        } else {
            self.stream.write_all(frame.serialize().as_slice())?;
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
            OPCODE::CLOSE => {
                self.connection_closed = true; 
                // todo!("Not implemented handle CLOSE frame");
                // TODO: If the client start the close handshake nothing to do
                // TODO: If the server start the close handsahke handle the response.
                println!("[CLIENT]: Connection close received")
            },
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
        let close_frame = ControlFrame::new(header, Some(status_code), msg.as_bytes().to_vec());

        // Add close frame into the queue
        self.event_queue.push_back((EventType::SEND, Box::new(close_frame)));

        let timeout = Instant::now();
        // Send response for all messages in the queue
        // TODO: Add timeout
        // TODO: Check if the boolean condition is ok.
        while !self.event_queue.is_empty() || !self.connection_closed {
            self.event_loop();
        }
            
        // At this point the client could receive messages from the server
        // TODO: Use read function from the client to read from the socket
        // TODO: Read the maximun size of a control frame to create an array of this size
        // TODO: Wait until close frame is received
        // let timeout = Instant::now();
        // loop {
        //     if timeout.elapsed().as_secs() >= DEFAULT_MESSAGE_SIZE { 
        //         println!("[CLIENT CLOSE]: Close handshake timeout, no response from server received.");
        //         break 
        //     }
        //     let response = read_frame(&mut self.stream, &mut self.recv_storage);

        //     match response {
        //         Ok(frame) => {
        //             if frame.is_none() { continue }
        //             let frame = frame.unwrap();

        //             match frame.kind() {
        //                 FrameKind::Data => 
        //                     if self.response_cb.is_some() { 
        //                     let res = String::from_utf8(frame.get_data().to_vec());

        //                 },
        //                 FrameKind::Control => self.handle_control_frame(frame.as_any().downcast_ref::<ControlFrame>().unwrap()).unwrap(),
        //                 FrameKind::NotDefine => continue
        //             }

        //             let data = frame.get_data();
        //             let msg =  str::from_utf8(&data[2..data.len()]).unwrap();
        //             println!("[CLIENT CLOSE]: Response from server -> {}", msg);
        //         },
        //         Err(_) => { }
        //     }
        // }

        // Close the socket.
        self.stream.shutdown(Shutdown::Both);
    }
}