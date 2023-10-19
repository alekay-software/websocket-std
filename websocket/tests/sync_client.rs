use std::net::{TcpListener, TcpStream};
use websocket_std::client::{sync_connect, SyncClient};
use websocket_std::result::{WebSocketError, WebSocketResult};
use std::thread;
use std::time::Duration;
use std::io::{self, Write, Read, ErrorKind};
use std::net::Shutdown;
use core::array::TryFromSliceError;
use std::sync::Arc;


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

fn mock_accept_connection(listener: TcpListener) -> TcpStream {
    let (mut conn, _) = listener.accept().unwrap();
    conn.set_nonblocking(true).unwrap();
    
    let _ = read_all(&mut conn).unwrap();
    
    let http_response = "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\n Connection: Upgrade\r\n\r\n".as_bytes();
    conn.write_all(http_response).unwrap();

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
    while _data.len() == 0 {
        let _data = read_all(conn).unwrap();
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
fn connection_success_no_close_handshake() {
    let (listener, port) = setup();
    
    thread::spawn(move || {
        let conn = mock_accept_connection(listener);
        conn.shutdown(Shutdown::Both).unwrap();
    });

    let connection = sync_connect("localhost", port, "/", None);
    let mut client: SyncClient<'static, u32> = connection.unwrap();
    client.set_timeout(Duration::from_secs(1));
}
#[test]
fn connection_error_no_server_running() {
    let (listener, _) = setup();
    
    thread::spawn(move || {
        let conn = mock_accept_connection(listener);
        conn.shutdown(Shutdown::Both).unwrap();
    });

    let connection: WebSocketResult<SyncClient<'static, u32>> = sync_connect("localhost", 0, "/", None);
    assert!(connection.is_err());
}

#[test]
fn mock_hanshake_error_unsuported_ws_version() {
    let (listener, port) = setup();
    
    thread::spawn(move || {
        let response = "HTTP/1.1 400 Bad Request\r\nDate: Thu, 07 Sep 2023 09:59:36 GMT\r\nServer: Python/3.9 websockets/11.0.3\r\nContent-Length: 80\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n".as_bytes();
        let conn = mock_refuse_connection(listener, response);
        conn.shutdown(Shutdown::Both).unwrap();
    });

    let connection: WebSocketResult<SyncClient<'static, u32>> = sync_connect("localhost", port, "/", None);
    assert!(connection.is_err());
    match connection.err().unwrap() {
        WebSocketError::HandShakeError(_) => assert!(true),
        _ => assert!(false, "Expected HandShakeError")
    }
}

#[test]
fn mock_hanshake_error_invalid_header() {
    let (listener, port) = setup();
    
    thread::spawn(move || {
        let response = "HTTP/1.1 400 Bad Request\r\nDate: Thu, 07 Sep 2023 10:06:58 GMT\r\nServer: Python/3.9 websockets/11.0.3\r\nContent-Length: 78\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\nFailed to open a WebSocket connection: invalid Sec-WebSocket-Key header: clo.\r\n\r\n".as_bytes();
        let conn = mock_refuse_connection(listener, response);
        conn.shutdown(Shutdown::Both).unwrap();
    });

    let connection: WebSocketResult<SyncClient<'static, u32>> = sync_connect("localhost", port, "/", None);
    assert!(connection.is_err());
    match connection.err().unwrap() {
        WebSocketError::HandShakeError(_) => assert!(true),
        _ => assert!(false, "Expected HandShakeError")
    }
}


// -------------------- Sending data -------------------- //
#[test]
fn send_data_success_on_one_frame() {
    fn callback(_ws: &mut SyncClient<u32>, msg: String, _data: Option<Arc<u32>>) {
        assert_eq!(msg, String::from("Hello"));
    }

    let (listener, port) = setup();
    
    thread::spawn(move || {
        let mut conn = mock_accept_connection(listener);
        let data = mock_wait_for_frame(&mut conn);
        let data = mock_unmask_data(&data);
        
        assert_eq!(String::from_utf8(data).unwrap(), "Hello");
        
        let echo_frame: Vec<u8> = [0x81, 0x05, 0x48, 0x65, 0x6c, 0x6c, 0x6f].to_vec();

        conn.write_all(echo_frame.as_slice()).unwrap();
        conn.shutdown(Shutdown::Both).unwrap();

    });

    let connection = sync_connect("localhost", port, "/", None);
    let mut client = connection.unwrap();
    client.set_timeout(Duration::from_secs(1));
    client.set_response_cb(callback, None);

    client.send("Hello").unwrap();

    let mut i = 0;
    while i < 2 {
        client.event_loop().unwrap();
        i += 1;
    }

}

#[test]
fn send_data_success_more_than_one_frame() {
    fn callback(_ws: &mut SyncClient<u32>, msg: String, _data: Option<Arc<u32>>) {
        assert_eq!(msg, String::from("Hello"));
    }

    let (listener, port) = setup();
    
    thread::spawn(move || {
        let mut conn = mock_accept_connection(listener);
        let mut msg = String::new();

        let mut i = 0;
        while i < 2 {
            let data = mock_wait_for_frame(&mut conn);
            let data = mock_unmask_data(&data);
            msg.push_str(String::from_utf8(data).unwrap().as_str());
            i += 1;
        }
        
        assert_eq!(msg, "Hello");
        
        let echo_frame: Vec<u8> = [0x81, 0x05, 0x48, 0x65, 0x6c, 0x6c, 0x6f].to_vec();

        conn.write_all(echo_frame.as_slice()).unwrap();
        conn.shutdown(Shutdown::Both).unwrap();

    });

    let connection = sync_connect("localhost", port, "/", None);
    let mut client = connection.unwrap();
    client.set_timeout(Duration::from_secs(1));
    client.set_message_size(3);
    client.set_response_cb(callback, None);

    client.send("Hello").unwrap();

    let mut i = 0;
    while i < 3 {
        client.event_loop().unwrap();
        i += 1;
    }

}

#[test]
fn connect_send_and_client_close_successfully() {
    fn callback(_ws: &mut SyncClient<u32>, msg: String, _data: Option<Arc<u32>>) {
        assert_eq!(msg, String::from("Hello"));
    }

    let (listener, port) = setup();
    
    thread::spawn(move || {
        let mut conn = mock_accept_connection(listener);
        let data = mock_wait_for_frame(&mut conn);
        let data = mock_unmask_data(&data);
        
        assert_eq!(String::from_utf8(data).unwrap(), "Hello");
        
        let echo_frame: Vec<u8> = [0x81, 0x05, 0x48, 0x65, 0x6c, 0x6c, 0x6f].to_vec();

        conn.write_all(echo_frame.as_slice()).unwrap();

        let close_frame = mock_wait_for_frame(&mut conn);
        let (status, reason) = mock_unmask_control_frame(&close_frame);
        
        assert_eq!(status, 1000);
        assert_eq!(String::from_utf8(reason).unwrap().as_str(), "Done");

        conn.shutdown(Shutdown::Both).unwrap();

    });

    let connection = sync_connect("localhost", port, "/", None);
    let mut client = connection.unwrap();
    client.set_timeout(Duration::from_secs(1));
    client.set_response_cb(callback, None);

    client.send("Hello").unwrap();
    assert!(client.is_connected());
    client.event_loop().unwrap();
    assert!(client.is_connected());

    drop(client);
}

// Test no cb set for response

// Test connection closed close frame not received
#[test]
fn server_close_connection_and_no_close_frame_received() {
    
}

// Test connection closed by client

// Test connection closed by the server

// Test Control frames can be interjected in the middle of a fragmented message.

// Test accept protocol