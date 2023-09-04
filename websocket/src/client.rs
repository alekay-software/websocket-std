use std::net::{TcpStream, Shutdown};
use std::io::{BufReader, BufRead, Read, Write};
use std::collections::{HashMap, VecDeque};
use std::time::Instant;
use std::format;
use crate::result::WebSocketError;
use crate::ws_basic::mask;
use crate::ws_basic::header::{Header, OPCODE, FLAG};
use crate::ws_basic::frame::{DataFrame, ControlFrame, Frame, FrameKind, read_frame};
use crate::core::traits::Serialize;
use super::result::WebSocketResult;
use crate::http::request::{Request, Method};

const DEFAULT_MESSAGE_SIZE: u64 = 1024;
const DEFAULT_TIMEOUT_SECS: u64 = 30;

#[derive(PartialEq)]
enum EventType {
    SEND,
    RECEIVED
}

#[allow(non_camel_case_types)]
#[derive(PartialEq)]
enum ConnectionStatus { 
    OPEN,
    CLOSED_BY_CLIENT,
    CLOSED_BY_SERVER
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

// [] TODO: Cerrar el socket cuando la conexion se ha cerrado por alguno de los 2 puntos y la cola de mensajes esta vacia.
// [] TODO: Event loop debe dar error cuando al conexion esta cerrada y todos los mensajes enviados.
// [] TODO: Implement framable trait (trait to split the data into frames)
// [] TODO: Create a trait to send and receive data from the websocket
// [x] TODO: Queues for messages to send
// [] TODO: Queues for messages to receive
// [x] TODO: Event loop must send messages from the queues
// [] TODO: Event loop must receive messages from the queues
// [] TODO: Decide if write or read messages
// [] TODO: Send the size of the buffer to read data from the stream, so the client will decide the perfomance base on the memory available or the size of the messages that the system is going to receive
// Remove warning dead code for [host, port, path] fields. The Client keeps this information because could be useful in the future.
#[allow(dead_code)]
pub struct SyncClient<'a> {
    host: &'a str,
    port: u16,
    path: &'a str,
    connection_status: ConnectionStatus,
    message_size: u64,
    response_cb: Option<fn(String)>,
    stream: TcpStream,
    event_queue: VecDeque<(EventType, Box<dyn Frame>)>,      // Events that the event loop will execute
    recv_storage: Vec<u8>,                                   // Storage to keep the bytes received from the socket
    send_queue: VecDeque<Box<dyn Frame>>,                    // Store frames to send                                
    recv_data: Vec<u8>                                       // Store the data received from the Frames until the data is completelly received
}

// TODO: No se implementa la funcion de cierre de la conexion, la conexion se cierra cuando termina la vida del cliente
// TODO: No hace falta comprobar los casos en los que el cliente cierra la conexion porque nunca va a llegar ese punto ocurre en su borrado de memoria
impl<'a> SyncClient<'a> {
    fn new(host: &'a str, port: u16, path: &'a str, stream: TcpStream) -> Self {
        SyncClient { host, port, path, connection_status: ConnectionStatus::OPEN, message_size: DEFAULT_MESSAGE_SIZE, response_cb: None, stream, event_queue: VecDeque::new(), recv_storage: Vec::new(), send_queue: VecDeque::new(), recv_data: Vec::new() }
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
        // Connection was closed
        if self.connection_status != ConnectionStatus::OPEN { 
            let msg = match self.connection_status {
                ConnectionStatus::CLOSED_BY_CLIENT => String::from("Connection closed by client"),
                ConnectionStatus::CLOSED_BY_SERVER => String::from("Connection closed by server"),
                ConnectionStatus::OPEN => String::from("")
            };
            return Err(WebSocketError::ConnectionClose(msg)) // The connection was closed, no more data can be send
        } 

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
        // TODO: Stop reading frames from the socket if the client closed the connection
        // (self.connection_status == ConnectionStatus::OPEN || self.connection_status == ConnectionStatus::CLOSED_BY_SERVER) 


        // Try to read Frames from the socket
        let frame = read_frame(&mut self.stream, &mut self.recv_storage)?;
        if frame.is_some() { self.event_queue.push_back((EventType::RECEIVED, frame.unwrap())); }

        // Insert more frames from the send queue
        let frame = self.send_queue.pop_front();
        if frame.is_some() { self.event_queue.push_back((EventType::SEND, frame.unwrap())); }

        // Take one frame from the eve  nt loop queue
        let res = self.event_queue.pop_front();
        if res.is_none() { return Ok(()) }  // No events to handle

        // Frame to handle
        let (event_type, frame) = res.unwrap();

        // If the client close the connection the received frames 
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
                            
                            // There is 2 ways to deal with the vector data:
                            // 1 - Remove from memory (takes more time)
                            //         Creating a new vector produces that the old vector will be dropped (deallocating the memory)
                            self.recv_data = Vec::new();

                            // // 2 - Use the clear method (takes more memory because we never drop it)
                            // //         The vector does not remove memory that has already been allocated.
                            // self.recv_data.clear();
                        }
                    }
                },
                FrameKind::Control => { return self.handle_control_frame(frame.as_any().downcast_ref::<ControlFrame>().unwrap()); },
                FrameKind::NotDefine => return Err(WebSocketError::ProtocolError(String::from("OPCODE not supported")))
            };
        // EventType == SEND    
        } else {
            if frame.kind() == FrameKind::Control && frame.get_header().get_opcode() == OPCODE::CLOSE { 
                // The client wants to close the connection
                self.connection_status = ConnectionStatus::CLOSED_BY_CLIENT;
            }
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
                match self.connection_status {
                    ConnectionStatus::OPEN => {
                        // Check the status code 1000, 1001, 1002...

                        // Server wants to  close the connection
                        // Enqueue close frame to response to the server
                        let payload: &[u8] = frame.get_data();
                        let header = Header::new(FLAG::FIN, OPCODE::CLOSE, Some(mask::gen_mask()), payload.len() as u64);
                        let close_frame = ControlFrame::new(header, Some(1000), payload.to_vec());
                        self.event_queue.push_back((EventType::SEND, Box::new(close_frame)));
                        
                        // Set connection status
                        self.connection_status = ConnectionStatus::CLOSED_BY_SERVER;
                        println!("Server wants to close the connection");
                    },
                    ConnectionStatus::CLOSED_BY_CLIENT => {}, // Nothing to do for now, we cou
                    ConnectionStatus::CLOSED_BY_SERVER => {}  // Unreachable  
                }
                println!("[CLIENT]: Connection close received")
            },
            _ => return Err(WebSocketError::ProtocolError(String::from("Invalid OPCODE for Control Frame")))
        }

        Ok(())
    }
}

// TODO: Refactor de code
impl<'a> Drop for SyncClient<'a> {
    fn drop(&mut self) {
        let msg = "Done";
        let mask = Some(mask::gen_mask());
        let header = Header::new(FLAG::FIN, OPCODE::CLOSE, mask, msg.len() as u64 + 2);
        let status_code: u16 = 1000;
        let close_frame = ControlFrame::new(header, Some(status_code), msg.as_bytes().to_vec());

        // Add close frame at the end of the queue.
        self.event_queue.push_back((EventType::SEND, Box::new(close_frame)));

        let timeout = Instant::now();
 
        // Process a response for all the events and confirm that the connection was closed.
        while !self.event_queue.is_empty() || self.connection_status != ConnectionStatus::OPEN {
            if timeout.elapsed().as_secs() >= DEFAULT_TIMEOUT_SECS { break } // Close handshake timeout.
            let result = self.event_loop();
            if result.is_ok() { continue }
            let err = result.err().unwrap();

            match err {
                WebSocketError::SocketError(_) => { break }, // If get an error with the socket, terminate the close handshake.
                _ => { continue }
            }

        }
        let _ = self.stream.shutdown(Shutdown::Both); // Ignore result from shutdown method.
    }
}