use std::net::{TcpStream, Shutdown};
use std::io::{Read, Write, ErrorKind};
use std::collections::{HashMap, VecDeque};
use std::time::{Instant, Duration};
use std::format;
use core::marker::Send;
use crate::result::WebSocketError;
use crate::ws_basic::header::{OPCODE, FLAG};
use crate::ws_basic::frame::{DataFrame, ControlFrame, Frame, FrameKind, read_frame};
use crate::ws_basic::status_code::{WSStatus, evaulate_status_code};
use crate::core::traits::{Serialize, Parse};
use crate::core::binary::bytes_to_u16;
use super::result::WebSocketResult;
use crate::http::request::{Request, Method};
use crate::http::response::Response;
use std::sync::Arc;
use crate::ws_basic::key::{gen_key, verify_key};

const DEFAULT_MESSAGE_SIZE: u64 = 1024;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const SWITCHING_PROTOCOLS: u16 = 101;

#[allow(non_camel_case_types)]
#[derive(PartialEq)]
enum ConnectionStatus { 
    OPEN,
    CLIENT_WANTS_TO_CLOSE,
    SERVER_WANTS_TO_CLOSE,
    CLOSE
}

// TODO: Confirm that the handshake is accepted
// TODO: Protocol and extensions
pub fn sync_connect<'a, T>(host: &'a str, port: u16, path: &'a str, subprotocols: Option<&[&str]>) -> WebSocketResult<SyncClient<'a, T>> {
    let mut socket = TcpStream::connect(format!("{}:{}", host, port.to_string()))?;

    let sec_websocket_key = gen_key();

    let mut headers: HashMap<&str, &str> = HashMap::from([
        ("Upgrade", "websocket"),
        ("Connection", "Upgrade"),
        ("Sec-WebSocket-Key", &sec_websocket_key),
        ("Sec-WebSocket-Version", "13"),
        ("User-agent", "rust-websocket-std"),
        ("Sec-WebSocket-Extensions", "person; name=ME_GUSTA_ESCUELA"),
    ]);

    let mut protocols_value = String::new();
    if let Some(protocols) = subprotocols {
        for p in protocols {
            protocols_value.push_str(p);
            protocols_value.push_str(", ");
        }
        headers.insert("Sec-WebSocket-Protocol", &(protocols_value)[0..protocols_value.len()-2]);
    }

    let request = Request::new(Method::GET, path, "HTTP/1.1", Some(headers));

    // Check result of write
    socket.write_all(request.serialize().as_slice())?;

    // Ensure that all data was sent
    socket.flush()?;

    // // TODO: Create a function to ensure that the entire http response was readed before try to parse it
    // // For the moment 512 bytes is enought to deal with connections
    let mut buffer = [0u8; 512];
    socket.read(&mut buffer)?;

    // Read response and verify that the server accepted switch protocols
    let response = Response::parse(&buffer);
    if response.is_err() { return Err(WebSocketError::HandShakeError(String::from("Error parsing response from server"))); }
    
    let response = response.unwrap();
    response.print();
    let sec_websocket_accept = response.header("Sec-WebSocket-Accept");

    if sec_websocket_accept.is_none() { return Err(WebSocketError::HandShakeError(String::from("No Sec-WebSocket-Accept received from server"))) }
    let sec_websocket_accept = sec_websocket_accept.unwrap();

    // Verify Sec-WebSocket-Accept
    let accepted = verify_key(&sec_websocket_key, &sec_websocket_accept);
    if !accepted {
        return Err(WebSocketError::HandShakeError(String::from("Invalid 'Sec-WebSocket-Accept'")));
    }

    if response.get_status_code() == 0 || response.get_status_code() != SWITCHING_PROTOCOLS { return Err(WebSocketError::HandShakeError(format!("HandShake Error: {}", String::from_utf8_lossy(&buffer)))) }

    // Set socket to non-blocking mode
    socket.set_nonblocking(true)?;
    
    let client = SyncClient::new(host, port, path, socket, response.header("Sec-WebSocket-Protocol"));

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
pub struct SyncClient<'a, T> {
    host: &'a str,
    port: u16,
    path: &'a str,
    connection_status: ConnectionStatus,
    message_size: u64,
    timeout: Duration,
    response_cb: Option<fn(&mut Self, String, Option<Arc<T>>)>,
    recv_frame_queue: VecDeque<Box<dyn Frame>>,              // Frames received queue
    send_frame_queue: VecDeque<Box<dyn Frame>>,              // Frames to send queue                               
    stream: TcpStream,
    recv_storage: Vec<u8>,                                   // Storage to keep the bytes received from the socket (bytes that didn't use to create a frame)
    recv_data: Vec<u8>,                                      // Store the data received from the Frames until the data is completelly received
    cb_data: Option<Arc<T>>,
    protocol: Option<String>,
    close_iters: usize,                                      // Count the number of times send_message tries to execute after the close. If <= 1 don't raise error, otherwise raise ConnectionClose error 
}                                                            // The close connection depends on the order of the functions event_loop and is_connected
                        

// TODO: No se implementa la funcion de cierre de la conexion, la conexion se cierra cuando termina la vida del cliente
// TODO: No hace falta comprobar los casos en los que el cliente cierra la conexion porque nunca va a llegar ese punto ocurre en su borrado de memoria
impl<'a, T> SyncClient<'a, T> {
    fn new(host: &'a str, port: u16, path: &'a str, stream: TcpStream, protocol: Option<String>) -> Self {
        SyncClient { 
            host, 
            port, 
            path, 
            connection_status: ConnectionStatus::OPEN, 
            message_size: DEFAULT_MESSAGE_SIZE, 
            response_cb: None, 
            stream, 
            recv_frame_queue: VecDeque::new(), 
            send_frame_queue: VecDeque::new(), 
            recv_storage: Vec::new(), 
            recv_data: Vec::new(), 
            timeout: DEFAULT_TIMEOUT, 
            cb_data: None,
            protocol,
            close_iters: 0
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connection_status != ConnectionStatus::CLOSE
    }

    // Returns the protocol accepted by the server
    pub fn protocol(&self) -> Option<&str> {
        if self.protocol.is_none() { return None };
        return Some(self.protocol.as_ref().unwrap().as_str());
    }

    // TODO: The message size does not take into account
    pub fn set_message_size(&mut self, size: u64) {
        self.message_size = size;
    }

    // TODO: This function is only for text messages, pass to the callback information about the type of the frame
    pub fn set_response_cb(&mut self, cb: fn(&mut Self, String, Option<Arc<T>>), data: Option<Arc<T>>) {
        self.response_cb = Some(cb);
        self.cb_data = data;
    }

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    // TODO: Create just one frame to send, if need to create more than one, store the rest of the bytes into a vector
    pub fn send(&mut self, payload: &str) -> WebSocketResult<()> {
        // Connection was closed
        if self.connection_status == ConnectionStatus::CLOSE {
            self.close_iters += 1;
            match self.connection_status {
                ConnectionStatus::CLOSE => {
                    if self.close_iters > 1 {
                        return Err(WebSocketError::ConnectionClose(String::from("Connection closed")));
                    } else {
                        return Ok(());
                    }
                }
                _ => return Ok(())
            };
        }

        let mut data_sent = 0;
        let mut _i: usize = 0;

        while data_sent < payload.len() {
            _i = data_sent + self.message_size as usize; 
            if _i >= payload.len() { _i = payload.len() };
            let payload_chunk = payload[data_sent.._i].as_bytes();
            let flag = if data_sent + self.message_size as usize >= payload.len() { FLAG::FIN } else { FLAG::NOFLAG };
            let code = if data_sent == 0 { OPCODE::TEXT } else { OPCODE::CONTINUATION };
            let frame = DataFrame::new(flag, code, payload_chunk.to_vec(), true, None);
            self.send_frame_queue.push_back(Box::new(frame));
            data_sent += self.message_size as usize;
        }

        Ok(())

    }

    // TODO: I'm assuming a lot of wrong things
    // TODO: What's happend if a close frame es received from the server?
    pub fn event_loop(&mut self) -> WebSocketResult<()> {
        // TODO: Stop reading frames from the socket if the client closed the connection
        // (self.connection_status == ConnectionStatus::OPEN || self.connection_status == ConnectionStatus::SERVER_WANTS_TO_CLOSE) 

        // To not read more frames if the connection is closed, check if there are data in the socket, if and EOF error is received then stop reading 
        // frames for future event iterations
        // If the error is raised check if the connection was closed by the client or the server

        // Try to read Frames from the socket
        let res = read_frame(&mut self.stream, &mut self.recv_storage);

        // Check if the stream is closed due the close handshake
        let mut frame = None;
        if res.is_err() {
            let e = res.err().unwrap();
            match e {
                WebSocketError::ConnectionClose(_) => {
                    if self.connection_status == ConnectionStatus::OPEN { return Err(e); }
                },
                _ => return Err(e)
            }
        } else {
            frame = res.unwrap();
        }

        if frame.is_some() { self.recv_frame_queue.push_back(frame.unwrap()); }


        // Take one frame to send
        let send_frame = self.send_frame_queue.pop_front();

        // Take one received frame
        let recv_frame = self.recv_frame_queue.pop_front();

        // If the client close the connection the received frames 
        if recv_frame.is_some() {
            let frame = recv_frame.unwrap();
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
                            callback(self, msg, self.cb_data.clone());

                        // Message from a multiples frames     
                        } else {
                            let previous_data = self.recv_data.clone();
                            let res = String::from_utf8(previous_data);
                            if res.is_err() { return Err(WebSocketError::Utf8Error(res.err().unwrap().utf8_error())); }
                            
                            let mut completed_msg = res.unwrap();
                            completed_msg.push_str(msg.as_str());

                            // Send the message to the callback function
                            callback(self, completed_msg, self.cb_data.clone());
                            
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
        } 
        
        if send_frame.is_some() {
            let frame = send_frame.unwrap();
            if frame.kind() == FrameKind::Control && frame.get_header().get_opcode() == OPCODE::CLOSE && self.connection_status == ConnectionStatus::OPEN { 
                self.connection_status = ConnectionStatus::CLIENT_WANTS_TO_CLOSE;
            } else if frame.kind() == FrameKind::Control && frame.get_header().get_opcode() == OPCODE::CLOSE && self.connection_status == ConnectionStatus::SERVER_WANTS_TO_CLOSE {
                self.connection_status = ConnectionStatus::CLOSE;
                println!("[CLIENT]: Sending response to close frame, status code ")       
            }

            self.try_write(frame)?;

            if self.connection_status == ConnectionStatus::CLOSE { 
                self.stream.shutdown(Shutdown::Both)?;
            }
        }
        
        Ok(())
    }

    fn try_write(&mut self, frame: Box<dyn Frame>) -> WebSocketResult<()> {
        let res = self.stream.write_all(frame.serialize().as_slice());
        if res.is_err(){
            let error = res.err().unwrap();

            // Try to send next iteration
            if error.kind() == ErrorKind::WouldBlock { 
                self.send_frame_queue.push_front(frame);

            } else {
                return Err(WebSocketError::IOError(error));
            }
        }
        Ok(())
    }

    fn handle_control_frame(&mut self, frame: &ControlFrame) -> WebSocketResult<()> {
        match frame.get_header().get_opcode() {
            OPCODE::PING=> { 
                let data = frame.get_data();
                let pong_frame = ControlFrame::new(FLAG::FIN, OPCODE::PONG, None, data.to_vec(), true, None);
                println!("[CLIENT]: Sending pong, data[{}]", data.len());
                self.try_write(Box::new(pong_frame))?;
            },
            OPCODE::PONG => { todo!("Not implemented handle PONG") },
            OPCODE::CLOSE => {
                let data = frame.get_data();
                let status_code = &data[0..2];
                let res = bytes_to_u16(status_code);

                let status_code = if res.is_ok() { res.unwrap() } else { WSStatus::EXPECTED_STATUS_CODE.bits() };

                match self.connection_status {
                    // Server wants to close the connection
                    ConnectionStatus::OPEN => {
                        println!("Server wants to close the connection");
                        let status_code = WSStatus::from_bits(status_code);

                        let reason = &data[2..data.len()];
                        let mut status_code = if status_code.is_some() { status_code.unwrap() } else { WSStatus::PROTOCOL_ERROR };
                        
                        let (error, _) = evaulate_status_code(status_code);
                        if error { status_code = WSStatus::PROTOCOL_ERROR }

                        // Enqueue close frame to response to the server
                        self.send_frame_queue.clear();
                        self.recv_frame_queue.clear();
                        let close_frame = ControlFrame::new(FLAG::FIN, OPCODE::CLOSE, Some(status_code.bits()), reason.to_vec(), true, None);
                        self.send_frame_queue.push_front(Box::new(close_frame));

                        println!("[RECEIVED STATUS]: {}", status_code.bits());
                        self.connection_status = ConnectionStatus::SERVER_WANTS_TO_CLOSE;
                        
                        // TODO: Create and on close cb to handle this situation, send the status code an the reason
                    },
                    ConnectionStatus::CLIENT_WANTS_TO_CLOSE => {
                        // TODO: ?
                        // Received a response to the client close handshake
                        // Verify the status of close handshake
                        self.connection_status = ConnectionStatus::CLOSE;
                        self.stream.shutdown(Shutdown::Both)?;
                    },
                    ConnectionStatus::SERVER_WANTS_TO_CLOSE => {}  // Unreachable  
                    ConnectionStatus::CLOSE => {}                  // Unreachable
                }
                println!("[CLIENT]: Connection close received")
            },
            _ => return Err(WebSocketError::ProtocolError(String::from("Invalid OPCODE for Control Frame")))
        }

        Ok(())
    }
}

// TODO: Refactor the code
impl<'a, T> Drop for SyncClient<'a, T> {
    fn drop(&mut self) {
        let msg = "Done";
        let status_code: u16 = 1000;
        let close_frame = ControlFrame::new(FLAG::FIN, OPCODE::CLOSE, Some(status_code), msg.as_bytes().to_vec(), true, None);

        // Add close frame at the end of the queue.
        // Clear both queues
        self.send_frame_queue.clear();
        self.recv_frame_queue.clear();
        self.send_frame_queue.push_back(Box::new(close_frame));

        let timeout = Instant::now();
 
        // Process a response for all the events and confirm that the connection was closed.
        while self.connection_status != ConnectionStatus::CLOSE {
            if timeout.elapsed().as_secs() >= self.timeout.as_secs() { break } // Close handshake timeout.
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

unsafe impl<'a, T> Send for SyncClient<'a, T> {}