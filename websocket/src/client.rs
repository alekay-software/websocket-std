use std::net::{TcpStream, Shutdown};
use std::io::{Write, ErrorKind};
use std::collections::{HashMap, VecDeque};
use std::time::{Instant, Duration};
use std::format;
use core::marker::Send;
use crate::core::net::read_into_buffer;
use crate::result::WebSocketError;
use crate::ws_basic::header::{OPCODE, FLAG};
use crate::ws_basic::frame::{DataFrame, ControlFrame, Frame, FrameKind, bytes_to_frame};
use crate::ws_basic::status_code::{WSStatus, evaulate_status_code};
use crate::core::traits::{Serialize, Parse};
use crate::core::binary::bytes_to_u16;
use super::result::WebSocketResult;
use crate::http::request::{Request, Method};
use crate::http::response::Response;
use std::sync::Arc;
use crate::ws_basic::key::{gen_key, verify_key};
use crate::extension::Extension;

const DEFAULT_MESSAGE_SIZE: u64 = 1024;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const SWITCHING_PROTOCOLS: u16 = 101;

#[allow(non_camel_case_types)]
#[derive(PartialEq)]
enum ConnectionStatus {
    NOT_INIT,
    HANDSHAKE, 
    OPEN,
    CLIENT_WANTS_TO_CLOSE,
    SERVER_WANTS_TO_CLOSE,
    CLOSE
}

#[allow(non_camel_case_types)]
enum Event {
    WEBSOCKET_DATA(Box<dyn Frame>),
    HTTP_RESPONSE(Response),
    HTTP_REQUEST(Request),
    NO_DATA,
}

enum EventIO {
    INPUT,
    OUTPUT
}

pub struct Config<'a, T> {
    pub on_connect: Option<fn(&mut SyncClient<'a, T>, Option<Arc<T>>)>, 
    pub on_data: Option<fn(&mut SyncClient<'a, T>, String, Option<Arc<T>>)>,
    pub on_close: Option<fn(Reason, Option<Arc<T>>)>,
    pub data: Option<Arc<T>>,
    pub protocols: Option<&'a[&'a str]>,
}

#[allow(non_camel_case_types)]
pub enum Reason {
    SERVER_CLOSE(u16),
    CLIENT_CLOSE(u16)
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
    on_data: Option<fn(&mut Self, String, Option<Arc<T>>)>, 
    on_connect: Option<fn(&mut Self, Option<Arc<T>>)>,                            
    on_close: Option<fn(Reason, Option<Arc<T>>)>,
    stream: Option<TcpStream>,
    recv_storage: Vec<u8>,                                   // Storage to keep the bytes received from the socket (bytes that didn't use to create a frame)
    recv_data: Vec<u8>,                                      // Store the data received from the Frames until the data is completelly received
    cb_data: Option<Arc<T>>,
    protocol: Option<String>,
    extensions: Vec<Extension>,
    input_events: VecDeque<Event>,
    output_events: VecDeque<Event>,
    websocket_key: String,
    close_iters: usize,                                      // Count the number of times send_message tries to execute after the close. If <= 1 don't raise error, otherwise raise ConnectionClose error 
}                                                            // The close connection depends on the order of the functions event_loop and is_connected
                        

// TODO: No se implementa la funcion de cierre de la conexion, la conexion se cierra cuando termina la vida del cliente
// TODO: No hace falta comprobar los casos en los que el cliente cierra la conexion porque nunca va a llegar ese punto ocurre en su borrado de memoria
impl<'a, T> SyncClient<'a, T> {
    pub fn new() -> Self {
        SyncClient { 
            host: "", 
            port: 0, 
            path: "", 
            connection_status: ConnectionStatus::NOT_INIT, 
            message_size: DEFAULT_MESSAGE_SIZE, 
            on_data: None,
            on_connect: None,
            on_close: None,
            stream: None, 
            recv_storage: Vec::new(), 
            recv_data: Vec::new(), 
            timeout: DEFAULT_TIMEOUT, 
            cb_data: None,
            protocol: None,
            extensions: Vec::new(),
            close_iters: 0,
            input_events: VecDeque::new(),
            output_events: VecDeque::new(),
            websocket_key: String::new(),
        }
    }

    pub fn init(&mut self, host: &'a str, port: u16, path: &'a str, config: Option<Config<'a, T>>) -> WebSocketResult<()> {
        
        let socket = TcpStream::connect(format!("{}:{}", host, port.to_string()));
        if socket.is_err() { return Err(WebSocketError::SocketError(String::from("Unreachable host"))); } 
        let sec_websocket_key = gen_key();
        
        let mut headers: HashMap<String, String> = HashMap::from([
            (String::from("Upgrade"), String::from("websocket")),
            (String::from("Connection"), String::from("Upgrade")),
            (String::from("Sec-WebSocket-Key"), sec_websocket_key.clone()),
            (String::from("Sec-WebSocket-Version"), String::from("13")),
            (String::from("User-agent"), String::from("rust-websocket-std")),
        ]);

        let mut protocols = None;

        if let Some(conf) = config {
            self.cb_data = conf.data;
            self.on_data = conf.on_data;
            self.on_close = conf.on_close;
            self.on_connect = conf.on_connect;
            protocols = conf.protocols;
        }

        // Add protocols to request
        let mut protocols_value = String::new();
        if let Some(protocols) = protocols {
            for p in protocols {
                protocols_value.push_str(p);
                protocols_value.push_str(", ");
            }
            headers.insert(String::from("Sec-WebSocket-Protocol"), (&(protocols_value)[0..protocols_value.len()-2]).to_string());
        }
        
        let request = Request::new(Method::GET, path, "HTTP/1.1", Some(headers));
        
        self.output_events.push_back(Event::HTTP_REQUEST(request));
        self.websocket_key = sec_websocket_key;
        self.connection_status = ConnectionStatus::HANDSHAKE;
        let socket = socket.unwrap();
        socket.set_nonblocking(true)?;
        self.stream = Some(socket);
            
        Ok(())
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

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    // TODO: Create just one frame to send, if need to create more than one, store the rest of the bytes into a vector
    pub fn send(&mut self, payload: &str) -> WebSocketResult<()> {
        // Connection was closed
        if self.connection_status == ConnectionStatus::NOT_INIT || self.connection_status == ConnectionStatus::HANDSHAKE {
            return Err(WebSocketError::SocketError(String::from("Client is not initialized")));
            // return Ok(());
        }
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
            self.output_events.push_back(Event::WEBSOCKET_DATA(Box::new(frame)));
            data_sent += self.message_size as usize;
        }

        Ok(())

    }

    pub fn event_loop(&mut self) -> WebSocketResult<()> {
        if self.connection_status == ConnectionStatus::NOT_INIT { return Ok(()) }
        if self.connection_status == ConnectionStatus::CLOSE { return Err(WebSocketError::ConnectionClose(String::from("Connection is closed"))) }
    
        let event = self.read_bytes_from_socket()?;
        self.insert_input_event(event);
 
        let in_event = self.input_events.pop_front();
        let out_event = self.output_events.pop_front();

        if in_event.is_some() { self.handle_event(in_event.unwrap(), EventIO::INPUT)? };
        if out_event.is_some() { self.handle_event(out_event.unwrap(), EventIO::OUTPUT)? };

        return Ok(())
    }

    fn handle_recv_bytes_frame(&mut self) -> WebSocketResult<Event> {
        let frame = bytes_to_frame(&self.recv_storage)?;
        if frame.is_none() { return Ok(Event::NO_DATA) };

        let (frame, offset) = frame.unwrap();

        let event = Event::WEBSOCKET_DATA(frame);
        self.recv_storage.drain(0..offset);

        Ok(event)
    }

    fn handle_recv_frame(&mut self, frame: Box<dyn Frame>) -> WebSocketResult<()> {
        match frame.kind()  {
            FrameKind::Data => { 
                if frame.get_header().get_flag() != FLAG::FIN {
                    self.recv_data.extend_from_slice(frame.get_data());
                }

                if self.on_data.is_some() {
                    let callback = self.on_data.unwrap();

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
                return Ok(());
            },
            FrameKind::Control => { return self.handle_control_frame(frame.as_any().downcast_ref::<ControlFrame>().unwrap()); },
            FrameKind::NotDefine => return Err(WebSocketError::ProtocolError(String::from("OPCODE not supported")))
        }; 
    }

    fn handle_recv_bytes_http_response(&mut self) -> WebSocketResult<Event> {
        let response = Response::parse(&self.recv_storage);
        if response.is_err() { return Ok(Event::NO_DATA); } // TODO: Check for timeout to raise an error

        let response = response.unwrap();
        let event = Event::HTTP_RESPONSE(response);
        // TODO: Drain bytes not used in response (maybe two responses comes at the same time)
        self.recv_storage.clear();

        Ok(event)
    }

    fn handle_recv_http_response(&mut self, response: Response) -> WebSocketResult<()> {
        match self.connection_status {
            ConnectionStatus::HANDSHAKE => {
                response.print();
                let sec_websocket_accept = response.header("Sec-WebSocket-Accept");
            
                if sec_websocket_accept.is_none() { return Err(WebSocketError::HandShakeError(String::from("No Sec-WebSocket-Accept received from server"))) }
                let sec_websocket_accept = sec_websocket_accept.unwrap();
            
                // Verify Sec-WebSocket-Accept
                let accepted = verify_key(&self.websocket_key, &sec_websocket_accept);
                if !accepted {
                    return Err(WebSocketError::HandShakeError(String::from("Invalid 'Sec-WebSocket-Accept'")));
                }
            
                if response.get_status_code() == 0 || 
                   response.get_status_code() != SWITCHING_PROTOCOLS { 
                    return Err(WebSocketError::HandShakeError(format!("HandShake Error: Server refused to upgrade connection to websockets"))) 
                }

                self.protocol = response.header("Sec-WebSocket-Protocol");

                self.connection_status = ConnectionStatus::OPEN;
                if let Some(on_connect) = self.on_connect {
                    on_connect(self, self.cb_data.clone());
                }
            }
            _ =>  {} // Unreachable 
        }

        Ok(())
    }

    fn handle_send_frame(&mut self, frame: Box<dyn Frame>) -> WebSocketResult<()> {
        let sent = self.try_write(frame.serialize().as_slice())?;
        let kind = frame.kind();
        let mut status = None;

        if frame.kind() == FrameKind::Control {
            status = frame.as_any().downcast_ref::<ControlFrame>().unwrap().get_status_code();
        }

        if !sent { self.output_events.push_front(Event::WEBSOCKET_DATA(frame)) };

        if sent && kind == FrameKind::Control && self.connection_status == ConnectionStatus::SERVER_WANTS_TO_CLOSE {
            self.connection_status = ConnectionStatus::CLOSE;
            self.stream.as_mut().unwrap().shutdown(Shutdown::Both)?;
            self.stream = None;

            if let Some(on_close) = self.on_close {
                on_close(Reason::SERVER_CLOSE(status.unwrap_or(0)), self.cb_data.clone());
            }
        }

        Ok(())
    }

    fn handle_send_http_request(&mut self, request: Request) -> WebSocketResult<()> {
        let sent = self.try_write(request.serialize().as_slice())?;
        if !sent { 
            self.output_events.push_front(Event::HTTP_REQUEST(request)) 
        }
        Ok(())
    }

    fn handle_event(&mut self, event: Event, kind: EventIO) -> WebSocketResult<()> {

        match kind {
            EventIO::INPUT => {
                match event {
                    Event::WEBSOCKET_DATA(frame) => self.handle_recv_frame(frame)?,
                    Event::HTTP_RESPONSE(response) => self.handle_recv_http_response(response)?,
                    Event::HTTP_REQUEST(_) => {} // Unreachable
                    Event::NO_DATA => {} // Unreachable
                }
            },

            EventIO::OUTPUT => {
                match event { 
                    Event::WEBSOCKET_DATA(frame) => self.handle_send_frame(frame)?,
                    Event::HTTP_REQUEST(request) => self.handle_send_http_request(request)?,
                    Event::HTTP_RESPONSE(_) => {} // Unreachable
                    Event::NO_DATA => {} // Unreachable
                }
            }
        }

        return Ok(());
    }

    fn read_bytes_from_socket(&mut self) -> WebSocketResult<Event> {
        // Add timeout attribute to self in order to raise an error if any op overflow the time required to finish
        let mut buffer = [0u8; 1024];
        let reader = self.stream.as_mut().unwrap();
        let bytes_readed = read_into_buffer(reader, &mut buffer)?;

        if bytes_readed > 0 {
            self.recv_storage.extend_from_slice(&buffer[0..bytes_readed]);
        }

        // Input data
        let mut event = Event::NO_DATA;
        if self.recv_storage.len() > 0 {
            match self.connection_status {
                ConnectionStatus::HANDSHAKE => event = self.handle_recv_bytes_http_response()?,
                ConnectionStatus::OPEN | ConnectionStatus::CLIENT_WANTS_TO_CLOSE | ConnectionStatus::SERVER_WANTS_TO_CLOSE => {
                    event = self.handle_recv_bytes_frame()?;
                },

                ConnectionStatus::CLOSE => {}, // Unreachable
                ConnectionStatus::NOT_INIT => {}, // Unreachable
            };
        }
        Ok(event) 
    }

    fn insert_input_event(&mut self, event: Event) {
        match &event {
            Event::WEBSOCKET_DATA(frame) => { 
                if frame.kind() == FrameKind::Control {
                    self.input_events.push_front(event);
                } else {
                    self.input_events.push_back(event)
                }
            },

            Event::HTTP_RESPONSE(_) => self.input_events.push_back(event),
            Event::HTTP_REQUEST(_) => {} // Unreachable
            Event::NO_DATA => {}
        }
    }

    fn try_write(&mut self, bytes: &[u8]) -> WebSocketResult<bool> {
        let res = self.stream.as_mut().unwrap().write_all(bytes);
        if res.is_err(){
            let error = res.err().unwrap();

            // Try to send next iteration
            if error.kind() == ErrorKind::WouldBlock { 
                return Ok(false);

            } else {
                return Err(WebSocketError::IOError(error));
            }
        }
        Ok(true)
    }

    fn handle_control_frame(&mut self, frame: &ControlFrame) -> WebSocketResult<()> {
        match frame.get_header().get_opcode() {
            OPCODE::PING=> { 
                let data = frame.get_data();
                let pong_frame = ControlFrame::new(FLAG::FIN, OPCODE::PONG, None, data.to_vec(), true, None);
                println!("[CLIENT]: Sending pong, data[{}]", data.len());
                self.output_events.push_front(Event::WEBSOCKET_DATA(Box::new(pong_frame)));
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
                        self.output_events.clear();
                        self.input_events.clear();
                        let close_frame = ControlFrame::new(FLAG::FIN, OPCODE::CLOSE, Some(status_code.bits()), reason.to_vec(), true, None);
                        self.output_events.push_front(Event::WEBSOCKET_DATA(Box::new(close_frame)));

                        println!("[RECEIVED STATUS]: {}", status_code.bits());
                        self.connection_status = ConnectionStatus::SERVER_WANTS_TO_CLOSE;
                        
                        // TODO: Create and on close cb to handle this situation, send the status code an the reason
                    },
                    ConnectionStatus::CLIENT_WANTS_TO_CLOSE => {
                        // TODO: ?
                        // Received a response to the client close handshake
                        // Verify the status of close handshake
                        self.connection_status = ConnectionStatus::CLOSE;
                        self.stream.as_mut().unwrap().shutdown(Shutdown::Both)?;
                        
                        if let Some(on_close) = self.on_close {
                            let f = frame.as_any().downcast_ref::<ControlFrame>().unwrap();
                            on_close(Reason::CLIENT_CLOSE(f.get_status_code().unwrap()), self.cb_data.clone());
                        }
                    },
                    ConnectionStatus::SERVER_WANTS_TO_CLOSE => {}  // Unreachable  
                    ConnectionStatus::CLOSE => {}                  // Unreachable
                    ConnectionStatus::HANDSHAKE => {}              // Unreachable
                    ConnectionStatus::NOT_INIT => {}               // Unreachable
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
        if self.connection_status != ConnectionStatus::NOT_INIT &&
           self.connection_status != ConnectionStatus::HANDSHAKE &&
           self.connection_status != ConnectionStatus::CLOSE &&
           self.stream.is_some() {

               let msg = "Done";
               let status_code: u16 = 1000;
               let close_frame = ControlFrame::new(FLAG::FIN, OPCODE::CLOSE, Some(status_code), msg.as_bytes().to_vec(), true, None);
       
               // Add close frame at the end of the queue.
               // Clear both queues
               self.output_events.clear();
               self.input_events.clear();
               self.output_events.push_back(Event::WEBSOCKET_DATA(Box::new(close_frame)));
               self.connection_status = ConnectionStatus::CLIENT_WANTS_TO_CLOSE;
       
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
               let _ = self.stream.as_mut().unwrap().shutdown(Shutdown::Both); // Ignore result from shutdown method.
        }
    }
}

unsafe impl<'a, T> Send for SyncClient<'a, T> {}