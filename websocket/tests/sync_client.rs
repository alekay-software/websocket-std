use std::net::{TcpListener, TcpStream};
use websocket_std::sync::client::{Config, Reason, WSEvent, WSClient};
use websocket_std::result::WebSocketError;
use std::thread;
use std::time::Duration;
use std::io::{self, Write, Read, ErrorKind};
use std::net::Shutdown;
use core::array::TryFromSliceError;
use std::sync::{Arc, RwLock};
use base64;
use sha1_smol::Sha1;
use std::rc::Rc;
use std::cell::RefCell;

// Globally Unique Identifier
const GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

// Returns the server TcpStream
fn setup() -> (TcpListener, u16) {
    let listener = TcpListener::bind("localhost:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    (listener, port)
}

fn read_all(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut buff: [u8; 1024] = [0; 1024];

    loop {
        let res = stream.read(&mut buff);
        match res {
            Ok(amount) => {
                let d = &(buff[0..amount]);
                data.extend_from_slice(d);
            }
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock { break }
                return Err(e);
            }
        }
    }
    
    return Ok(data);
}

fn read_all_sync(stream: &mut TcpStream) -> Vec<u8> {
    let mut buff: [u8; 1024] = [0; 1024];
    let mut data = Vec::new();
    match stream.read(&mut buff) {
        Ok(amount) => { data.extend_from_slice(&buff[0..amount]) },
        Err(e) => {}
    }

    return data;
}

fn sec_websocket_accept(sec_websocket_key: &str) -> String {
    let mut accept_key = String::with_capacity(sec_websocket_key.len() + GUID.len());
    accept_key.push_str(sec_websocket_key);
    accept_key.push_str(GUID);
    let mut hasher = Sha1::new();
    hasher.update(accept_key.as_bytes());
    let accept_key = hasher.digest().bytes();
    let accept_key = base64::encode(&accept_key);
    return accept_key;
}

fn mock_accept_connection_no_websocket_key(listener: TcpListener) -> TcpStream {
    let (mut conn, _) = listener.accept().unwrap();
    
    let _ = read_all_sync(&mut conn);
    
    let http_response = "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\n Connection: Upgrade\r\n\r\n".as_bytes();
    conn.write_all(http_response).unwrap();

    return conn;
}

fn mock_accept_connection(listener: TcpListener) -> TcpStream {
    let (mut conn, _) = listener.accept().unwrap();
    
    let request = read_all_sync(&mut conn);
    print!("{}\n", String::from_utf8(request.clone()).unwrap());

    let request = String::from_utf8(request).unwrap();
    let request_lower = request.replace(" ", "").to_lowercase();
    let mut i = request_lower.find("sec-websocket-key").unwrap();
    let mut key = request.get(i..request.len()).unwrap();
    i = key.find(":").unwrap();
    key = key.get(i+1..key.len()).unwrap();
    i = key.find("\r\n").unwrap();
    key = key.get(0..i).unwrap();
    key = key.trim_start();

    print!("key: {}\n", key);

    let accept_key = sec_websocket_accept(key);

    let http_resonse = format!("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\n Connection: Upgrade\r\n sec-websocket-accept: {}\r\n\r\n", accept_key);

    conn.write_all(http_resonse.as_bytes()).unwrap();

    return conn;
}

fn mock_refuse_connection(listener: TcpListener, http_response: &[u8]) -> TcpStream {
    let (mut conn, _) = listener.accept().unwrap();
    conn.set_nonblocking(true).unwrap();
    
    let _ = read_all(&mut conn).unwrap();
    conn.write_all(http_response).unwrap();

    return conn;
}

fn mock_wait_for_frame(conn: &mut TcpStream) -> Vec<u8> {
    let _data: Vec<u8> = Vec::new();
    let mut data_res = Vec::new();
    while data_res.len() == 0 {
        let _data = read_all(conn).unwrap();
        data_res.extend(_data);
    }
    return data_res;
}

fn mock_wait_for_frame_sync(conn: &mut TcpStream) -> Vec<u8> {
    let _data: Vec<u8> = Vec::new();
    let mut data_res = Vec::new();
    while data_res.len() == 0 {
        let _data = read_all_sync(conn);
        data_res.extend(_data);
    }
    return data_res;
}

fn mock_unmask_data(data: &Vec<u8>) -> Vec<u8> {
    let bytes = data.as_slice();
    let mask = &bytes[2..6];
    let masked_data = &bytes[6..bytes.len()];

    let mut data = Vec::new();
        
    let mut i = 0;
    for &byte in masked_data {
        data.push(byte ^ mask[i]);
        i += 1;
        if i >= mask.len() { i = 0 };
    }
    data
}

pub fn bytes_to_u16(bytes: &[u8]) -> Result<u16, TryFromSliceError> {
    let res: Result<[u8; 2], _> = bytes.try_into();
    if res.is_err() { return Err(res.err().unwrap()); }
    let buf = res.unwrap();
    return Ok(u16::from_be_bytes(buf));
}

fn mock_unmask_control_frame(data: &Vec<u8>) -> (u16, Vec<u8>) {
    let bytes = data.as_slice();
    let mask = &bytes[2..6];
    let masked_status = &bytes[6..8];
    let masked_reason = &bytes[8..bytes.len()];

    let mut reason: Vec<u8> = Vec::new();
    let mut status: Vec<u8> = Vec::new();

    let mut i = 0;
    for &byte in masked_reason {
        reason.push(byte ^ mask[i]);
        i += 1;
        if i >= mask.len() { i = 0 };
    }

    i = 0;
    for &byte in masked_status {
        status.push(byte ^ mask[i]);
        i += 1;
        if i >= mask.len() { i = 0 };
    }

    let status = bytes_to_u16(status.as_slice()).unwrap();
    
    (status, reason)
}

// -------------------- Connection and HandShake -------------------- //

#[test]
fn connection_success_handshake_error_no_sec_websocket_received() {

    struct Data {
        connected: bool
    }
    
    type WSData = Arc<RwLock<Data>>;
    type WebSocket<'a> = WSClient<'a, WSData>;
    let data: WSData = Arc::new(RwLock::new(Data { connected: false }));

    let data_client= data.clone();

    let (listener, port) = setup();
    
    thread::spawn(move || {
        let conn = mock_accept_connection_no_websocket_key(listener);
        while !data_client.read().unwrap().connected {}
        conn.shutdown(Shutdown::Both).unwrap();
    });

    fn websocket_handler(_ws: &mut WebSocket, event: &WSEvent, _data: Option<WSData>) {
        match event {
            WSEvent::ON_CONNECT(_) => {},
            WSEvent::ON_TEXT(_) => {},
            WSEvent::ON_CLOSE(_) => {} 
        }
    } 

    let config  = Config { 
        callback: Some(websocket_handler),
        data: Some(data.clone()),
        protocols: None 
    };

    let config: Option<Config<WSData>> = Some(config);
    let mut client = WSClient::new();
    client.set_timeout(Duration::from_secs(1));
    client.init("localhost", port, "/", config);

    while !data.read().unwrap().connected {
        match client.event_loop() {
            Ok(_) => {},
            Err(e) => {
                assert!(e == WebSocketError::HandShake);
                break;
            }
        }
    }

    assert!(!data.read().unwrap().connected);

}


// #[test]
// fn connection_success_no_close_handshake() {

//     struct Data {
//         connected: bool
//     }
    
//     type WSData = Arc<RwLock<Data>>;
//     type WebSocket<'a> = WSClient<'a, WSData>;
//     let data: WSData = Arc::new(RwLock::new(Data { connected: false }));

//     let data_client= data.clone();

//     let (listener, port) = setup();
    
//     thread::spawn(move || {
//         let conn = mock_accept_connection(listener);
//         while !data_client.read().unwrap().connected {}
//         conn.shutdown(Shutdown::Both).unwrap();
//     });

//     fn websocket_handler(ws: &mut WebSocket, event: &WSEvent, data: Option<WSData>) {
//         match event {
//             WSEvent::ON_CONNECT(msg) => on_connect(ws, msg, data),
//             WSEvent::ON_TEXT(msg) => on_message(ws, msg, data),
//             WSEvent::ON_CLOSE(reason) => on_close(reason, data)
//         }
//     } 

//     fn on_message(ws: &mut WebSocket, msg: &String, data: Option<WSData>) {}

//     fn on_close(reason: &Reason, _data: Option<WSData>) {
//         match reason {
//             Reason::CLIENT_CLOSE(_) => assert!(true),
//             Reason::SERVER_CLOSE(_) => assert!(false)
//         }
//     }

//     fn on_connect(_ws: &mut WebSocket, msg: &Option<String>, data: Option<WSData>) {
//         let d = data.unwrap();
//         let mut d = d.write().unwrap();
//         d.connected = true;
//     }

//     let config  = Config { 
//         callback: Some(websocket_handler),
//         data: Some(data.clone()),
//         protocols: None 
//     };

//     let config: Option<Config<WSData>> = Some(config);
//     let mut client = WSClient::new();
//     client.set_timeout(Duration::from_secs(1));
//     client.init("localhost", port, "/", config);

//     while !data.read().unwrap().connected {
//         match client.event_loop() {
//             Ok(_) => {},
//             Err(e) => {
//                 assert!(e == WebSocketError::ConnectionClose);
//                 break;
//             } 
//         }
//     }

//     assert!(data.read().unwrap().connected);
// }

// #[test]
// fn connection_error_no_server_running() {
//     let (listener, _) = setup();
    
//     thread::spawn(move || {
//         let conn = mock_accept_connection(listener);
//         conn.shutdown(Shutdown::Both).unwrap();
//     });

//     let connection: WebSocketResult<SyncClient<'static, u32>> = sync_connect("localhost", 0, "/", None);
//     assert!(connection.is_err());
// }

// #[test]
// fn mock_hanshake_error_unsuported_ws_version() {
//     let (listener, port) = setup();
    
//     thread::spawn(move || {
//         let response = "HTTP/1.1 400 Bad Request\r\nDate: Thu, 07 Sep 2023 09:59:36 GMT\r\nServer: Python/3.9 websockets/11.0.3\r\nContent-Length: 80\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n".as_bytes();
//         let conn = mock_refuse_connection(listener, response);
//         conn.shutdown(Shutdown::Both).unwrap();
//     });

//     let connection: WebSocketResult<SyncClient<'static, u32>> = sync_connect("localhost", port, "/", None);
//     assert!(connection.is_err());
//     match connection.err().unwrap() {
//         WebSocketError::HandShakeError(_) => assert!(true),
//         _ => assert!(false, "Expected HandShakeError")
//     }
// }

// #[test]
// fn mock_hanshake_error_invalid_header() {
//     let (listener, port) = setup();
    
//     thread::spawn(move || {
//         let response = "HTTP/1.1 400 Bad Request\r\nDate: Thu, 07 Sep 2023 10:06:58 GMT\r\nServer: Python/3.9 websockets/11.0.3\r\nContent-Length: 78\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\nFailed to open a WebSocket connection: invalid Sec-WebSocket-Key header: clo.\r\n\r\n".as_bytes();
//         let conn = mock_refuse_connection(listener, response);
//         conn.shutdown(Shutdown::Both).unwrap();
//     });

//     let connection: WebSocketResult<SyncClient<'static, u32>> = sync_connect("localhost", port, "/", None);
//     assert!(connection.is_err());
//     match connection.err().unwrap() {
//         WebSocketError::HandShakeError(_) => assert!(true),
//         _ => assert!(false, "Expected HandShakeError")
//     }
// }


// TODO: on_close is never executed because the mocked server never response to close handshake
// // -------------------- Sending data -------------------- //
#[test]
fn send_data_success_on_one_frame() {

    type WSData = Rc<RefCell<u32>>;
    type WebSocket<'a> = WSClient<'a, WSData>;
    let data: WSData = Rc::new(RefCell::new(0));

    let (listener, port) = setup();
    
    thread::spawn(move || {
        let mut conn = mock_accept_connection(listener);
        let data = mock_wait_for_frame_sync(&mut conn);
        let data = mock_unmask_data(&data);
        
        assert_eq!(String::from_utf8(data).unwrap(), "Hello");
        
        let echo_frame: Vec<u8> = [0x81, 0x05, 0x48, 0x65, 0x6c, 0x6c, 0x6f].to_vec();

        conn.write_all(echo_frame.as_slice()).unwrap();
        conn.shutdown(Shutdown::Both).unwrap();

    });

    fn websocket_handler(ws: &mut WebSocket, event: &WSEvent, data: Option<WSData>) {
        match event {
            WSEvent::ON_CONNECT(_) => {},
            WSEvent::ON_TEXT(msg) => on_message(ws, msg, data),
            WSEvent::ON_CLOSE(reason) => on_close(reason, data)
        }
    } 

    fn on_message(ws: &mut WebSocket, msg: &String, data: Option<WSData>) {
        assert!(msg == "Hello");
        let d = data.unwrap();
        let mut d = d.borrow_mut();
        *d += 1;
    }

    fn on_close(reason: &Reason, _data: Option<WSData>) {
        match reason {
            Reason::CLIENT_CLOSE(_) => assert!(true),
            Reason::SERVER_CLOSE(_) => assert!(false)
        }
    }

    let config  = Config { 
        callback: Some(websocket_handler),
        data: Some(data.clone()),
        protocols: None 
    };

    let config: Option<Config<WSData>> = Some(config);
    let mut client = WSClient::new();
    client.set_timeout(Duration::from_secs(10));
    client.init("localhost", port, "/", config);
    client.send("Hello");

    while *data.borrow() < 1 {
        client.event_loop().unwrap();
    }

    assert!(*data.borrow() == 1);

}

#[test]
fn send_data_success_more_than_one_frame() {
    type WSData = Rc<RefCell<u32>>;
    type WebSocket<'a> = WSClient<'a, WSData>;
    let data: WSData = Rc::new(RefCell::new(0));

    let (listener, port) = setup();
    
    thread::spawn(move || {
        let mut conn = mock_accept_connection(listener);
        let mut msg = String::new();

        for _ in 0..2 {
            let data = mock_wait_for_frame_sync(&mut conn);
            let data = mock_unmask_data(&data);
            match String::from_utf8(data) {
                Ok(d) => msg.push_str(d.as_str()),
                Err(e) => { print!("error: {}", e)}
            }
        }
        
        assert_eq!(msg, "Hello");
        
        let echo_frame: Vec<u8> = [0x81, 0x05, 0x48, 0x65, 0x6c, 0x6c, 0x6f].to_vec();

        conn.write_all(echo_frame.as_slice()).unwrap();
        conn.shutdown(Shutdown::Both).unwrap();

    });

    fn websocket_handler(ws: &mut WebSocket, event: &WSEvent, data: Option<WSData>) {
        match event {
            WSEvent::ON_CONNECT(msg) => {},
            WSEvent::ON_TEXT(msg) => on_message(ws, msg, data),
            WSEvent::ON_CLOSE(reason) => on_close(reason, data)
        }
    } 

    fn on_message(_ws: &mut WebSocket, _: &String, data: Option<WSData>) {
        let d = data.unwrap();
        let mut d = d.borrow_mut();
        *d +=1; 
    }

    fn on_close(reason: &Reason, _data: Option<WSData>) {
        match reason {
            Reason::CLIENT_CLOSE(_) => assert!(true),
            Reason::SERVER_CLOSE(_) => assert!(false)
        }
    }

    let config  = Config { 
        callback: Some(websocket_handler),
        data: Some(data.clone()),
        protocols: None 
    };

    let config: Option<Config<WSData>> = Some(config);
    let mut client = WSClient::new();
    client.set_timeout(Duration::from_secs(1));
    client.set_message_size(3);
    client.init("localhost", port, "/", config);
    client.send("Hello");

    while *data.borrow() < 1 {
        client.event_loop().unwrap();
        thread::sleep(Duration::from_millis(50)); // Force the client to be slow in order to receive the data in two frames by the mocked server
    }

    drop(client);

    assert!(*data.borrow() == 1);
} 

#[test]
fn connect_send_and_client_close_successfully() {
    type WSData = Rc<RefCell<u32>>;
    type WebSocket<'a> = WSClient<'a, WSData>;
    let data: WSData = Rc::new(RefCell::new(0));

    let (listener, port) = setup();
    
    thread::spawn(move || {
        let mut conn = mock_accept_connection(listener);

        let data = mock_wait_for_frame_sync(&mut conn);
        let data = mock_unmask_data(&data);
        
        assert_eq!(String::from_utf8(data).unwrap(), "Hello");
        
        let echo_frame: Vec<u8> = [0x81, 0x05, 0x48, 0x65, 0x6c, 0x6c, 0x6f].to_vec();

        conn.write_all(echo_frame.as_slice()).unwrap();
        let close_frame = mock_wait_for_frame_sync(&mut conn);

        let (status, reason) = mock_unmask_control_frame(&close_frame);
        
        assert_eq!(status, 1000);
        assert_eq!(String::from_utf8(reason).unwrap().as_str(), "Done");

        conn.shutdown(Shutdown::Both).unwrap();

    });

    fn websocket_handler(ws: &mut WebSocket, event: &WSEvent, data: Option<WSData>) {
        match event {
            WSEvent::ON_CONNECT(msg) => {},
            WSEvent::ON_TEXT(msg) => on_message(ws, msg, data),
            WSEvent::ON_CLOSE(reason) => on_close(reason, data)
        }
    } 

    fn on_message(_ws: &mut WebSocket, _: &String, data: Option<WSData>) {
        let d = data.unwrap();
        let mut d = d.borrow_mut();
        *d +=1;
    }

    fn on_close(reason: &Reason, _data: Option<WSData>) {
        match reason {
            Reason::CLIENT_CLOSE(_) => assert!(true),
            Reason::SERVER_CLOSE(_) => assert!(false)
        }
    }

    let config  = Config { 
        callback: Some(websocket_handler),
        data: Some(data.clone()),
        protocols: None 
    };

    let config: Option<Config<WSData>> = Some(config);
    let mut client = WSClient::new();
    client.set_timeout(Duration::from_secs(1));
    client.init("localhost", port, "/", config);
    client.send("Hello");

    while *data.borrow() < 1 {
        client.event_loop().unwrap();
        thread::sleep(Duration::from_millis(50));
    }

    drop(client);

    assert!(*data.borrow() == 1);
}

// #[test]
// fn connect_send_and_client_close_successfully() {
//     fn callback(_ws: &mut SyncClient<u32>, msg: String, _data: Option<Arc<u32>>) {
//         assert_eq!(msg, String::from("Hello"));
//     }

//     let (listener, port) = setup();
    
//     thread::spawn(move || {
//         let mut conn = mock_accept_connection(listener);
//         let data = mock_wait_for_frame(&mut conn);
//         let data = mock_unmask_data(&data);
        
//         assert_eq!(String::from_utf8(data).unwrap(), "Hello");
        
//         let echo_frame: Vec<u8> = [0x81, 0x05, 0x48, 0x65, 0x6c, 0x6c, 0x6f].to_vec();

//         conn.write_all(echo_frame.as_slice()).unwrap();

//         let close_frame = mock_wait_for_frame(&mut conn);
//         let (status, reason) = mock_unmask_control_frame(&close_frame);
        
//         assert_eq!(status, 1000);
//         assert_eq!(String::from_utf8(reason).unwrap().as_str(), "Done");

//         conn.shutdown(Shutdown::Both).unwrap();

//     });

//     let connection = sync_connect("localhost", port, "/", None);
//     let mut client = connection.unwrap();
//     client.set_timeout(Duration::from_secs(1));
//     client.set_response_cb(callback, None);

//     client.send("Hello").unwrap();
//     assert!(client.is_connected());
//     client.event_loop().unwrap();
//     assert!(client.is_connected());

//     drop(client);
// }

// // Test no cb set for response

// // Test connection closed close frame not received
// #[test]
// fn server_close_connection_and_no_close_frame_received() {
    
// }

// // Test connection closed by client

// // Test connection closed by the server

// // Test Control frames can be interjected in the middle of a fragmented message.

// // Test accept protocol