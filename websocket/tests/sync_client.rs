use std::net::{TcpListener, TcpStream};
use websocket_std::client::sync_connect;
use std::thread;
use std::io::{self, Write, Read, ErrorKind};
use std::net::Shutdown;

// ----------- HandShake test -----------
// - TODO: Add test for supported version of http
// - TODO: Add test for supported version of websocket
// - TODO: Add test to check server connection upgrade accepted
//      HTTP/1.1 101 Switching Protocols
//      Upgrade: websocket
//      Connection: Upgrade


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

#[test]
fn connection_success() {
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