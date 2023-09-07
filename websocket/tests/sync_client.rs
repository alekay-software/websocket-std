use std::net::{TcpListener, TcpStream};
use websocket_std::client::sync_connect;
use websocket_std::result::WebSocketError;
use std::thread;
use std::io::{self, Write, Read, ErrorKind};
use std::net::Shutdown;

// Returns the server TcpStream
fn setup() -> (TcpListener, u16) {
    let listener = TcpListener::bind("localhost:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    (listener, port)
}

fn read_all(stream: &mut TcpStream) -> io::Result<String> {
    let mut data = String::new();
    let mut buff: [u8; 1024] = [0; 1024];

    loop {
        let res = stream.read(&mut buff);
        match res {
            Ok(amount) => {
                let d = String::from_utf8(buff[0..amount].to_vec()).unwrap();
                data.push_str(d.as_str());
            }
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock { break }
                return Err(e);
            }
        }
    }
    
    return Ok(data);
}

// This test lasts 30 seconds because the client is waiting for the handshake response
// The client by default waits 30 seconds to receive a response before closing.
#[test]
fn connection_success_no_close_handshake() {
    let (listener, port) = setup();
    
    thread::spawn(move || {
        let (mut conn, _) = listener.accept().unwrap();
        conn.set_nonblocking(true).unwrap();
        
        let _ = read_all(&mut conn).unwrap();
        
        // Response with switchin protocols
        let http_response = "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\n Connection: Upgrade\r\n\r\n".as_bytes();
        conn.write_all(http_response).unwrap();
        conn.shutdown(Shutdown::Both).unwrap();
    });

    let connection = sync_connect("localhost", port, "/");
    assert!(connection.is_ok());
}
#[test]
fn connection_error_no_server_running() {
    let (listener, port) = setup();
    
    thread::spawn(move || {
        let (mut conn, _) = listener.accept().unwrap();
        conn.set_nonblocking(true).unwrap();
        
        let _ = read_all(&mut conn).unwrap();
        
        // Response with switchin protocols
        let http_response = "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\n Connection: Upgrade\r\n\r\n".as_bytes();
        conn.write_all(http_response).unwrap();
        conn.shutdown(Shutdown::Both).unwrap();
    });

    let connection = sync_connect("localhost", port + 1, "/");
    assert!(connection.is_err());
}

#[test]
fn mock_hanshake_error_unsuported_ws_version() {
    let (listener, port) = setup();
    
    thread::spawn(move || {
        let (mut conn, _) = listener.accept().unwrap();
        conn.set_nonblocking(true).unwrap();
        
        let _ = read_all(&mut conn).unwrap();
        
        // Response with switchin protocols
        let response = "HTTP/1.1 400 Bad Request\r\nDate: Thu, 07 Sep 2023 09:59:36 GMT\r\nServer: Python/3.9 websockets/11.0.3\r\nContent-Length: 80\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n".as_bytes();

        conn.write_all(response).unwrap();
        conn.shutdown(Shutdown::Both).unwrap();
    });

    let connection = sync_connect("localhost", port + 1, "/");
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
        let (mut conn, _) = listener.accept().unwrap();
        conn.set_nonblocking(true).unwrap();
        
        let _ = read_all(&mut conn).unwrap();
        
        // Response with switchin protocols
        let response = "HTTP/1.1 400 Bad Request\r\nDate: Thu, 07 Sep 2023 10:06:58 GMT\r\nServer: Python/3.9 websockets/11.0.3\r\nContent-Length: 78\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\nFailed to open a WebSocket connection: invalid Sec-WebSocket-Key header: clo.\r\n\r\n".as_bytes();

        conn.write_all(response).unwrap();
        conn.shutdown(Shutdown::Both).unwrap();
    });

    let connection = sync_connect("localhost", port + 1, "/");
    assert!(connection.is_err());
    match connection.err().unwrap() {
        WebSocketError::HandShakeError(_) => assert!(true),
        _ => assert!(false, "Expected HandShakeError")
    }
}