use super::net::read_into_buffer;
use std::io::Write;
use std::net::{TcpStream, TcpListener, Shutdown};
use std::time::Duration;
use std::thread::sleep;
use crate::result::WebSocketError;

// -------------------------------------------------------------------------------------------------------- //
//                                               net.rs
// -------------------------------------------------------------------------------------------------------- //


// Setup server and client socket
// Client socket must be in not_blocking mode
// Test run in parallel so create socket with different ports to avoid errors taken the same address
fn setup() -> (TcpStream, TcpStream) {
    // With port 0 the program will request and available random port
    let listener = TcpListener::bind("localhost:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let client = TcpStream::connect(format!("localhost:{}", port)).unwrap();
    client.set_nonblocking(true).unwrap();
    let (server, _) = listener.accept().unwrap();

    (server, client)
}

fn before_each(server: TcpStream, client: TcpStream) {
    server.shutdown(Shutdown::Both);
    client.shutdown(Shutdown::Both);
}

#[test]
fn no_bytes_ready_to_read () {
    let (server, mut client) = setup();
    let mut buf: [u8; 8] = [0; 8];

    let res = read_into_buffer(&mut client, &mut buf);
    assert!(res.is_ok());
    let amount = res.unwrap();
    assert_eq!(0, amount);

    before_each(server, client);
}

#[test]
fn eof_reached () {
    let (server, mut client) = setup();
    let mut buf: [u8; 8] = [0; 8];

    server.shutdown(Shutdown::Both);
    sleep(Duration::from_secs(1));

    let res = read_into_buffer(&mut client, &mut buf);
    assert!(res.is_err());
    let error = res.err().unwrap();
    
    match error {
        WebSocketError::Custom(_) => assert!(true),
        e => panic!("Unreachable: {}", e) // grcov-excl-line 
    }

    client.shutdown(Shutdown::Both);
}

#[test]
fn read_less_than_buffer_capacity () {
    let (mut server, mut client) = setup();
    let mut buf: [u8; 8] = [0; 8];

    let msg = "hello";
    server.write_all(msg.as_bytes()).unwrap();
    sleep(Duration::from_secs(1));

    let res = read_into_buffer(&mut client, &mut buf);
    assert!(res.is_ok());
    let amount = res.unwrap();
    assert_eq!(msg.len(), amount);

    let res = read_into_buffer(&mut client, &mut buf);
    assert!(res.is_ok());
    let amount = res.unwrap();
    assert_eq!(0, amount);

    before_each(server, client);
}

#[test]
fn read_same_as_buffer_capacity () {
    let (mut server, mut client) = setup();
    let mut buf: [u8; 8] = [0; 8];

    let msg = "hello!!!";
    server.write_all(msg.as_bytes()).unwrap();
    sleep(Duration::from_secs(1));

    let res = read_into_buffer(&mut client, &mut buf);
    assert!(res.is_ok());
    let amount = res.unwrap();
    assert_eq!(msg.len(), amount);
    assert_eq!(msg.len(), buf.len());

    let res = read_into_buffer(&mut client, &mut buf);
    assert!(res.is_ok());
    let amount = res.unwrap();
    assert_eq!(0, amount);

    before_each(server, client);
}

#[test]
fn read_more_than_buffer_capacity () {
    let (mut server, mut client) = setup();
    let mut buf: [u8; 8] = [0; 8];

    let msg = "hello world!";
    server.write_all(msg.as_bytes()).unwrap();
    sleep(Duration::from_secs(1));

    let res = read_into_buffer(&mut client, &mut buf);
    assert!(res.is_ok());
    let amount = res.unwrap();
    assert_eq!(buf.len(), amount);
    assert!(msg.len() > buf.len());

    let res = read_into_buffer(&mut client, &mut buf);
    assert!(res.is_ok());
    let amount = res.unwrap();
    assert_eq!(msg.len() - buf.len(), amount);

    let res = read_into_buffer(&mut client, &mut buf);
    assert!(res.is_ok());
    let amount = res.unwrap();
    assert_eq!(0, amount);

    before_each(server, client);
}

// -------------------------------------------------------------------------------------------------------- //
//                                               binary.rs
// -------------------------------------------------------------------------------------------------------- //
use super::binary::*;

#[test]
fn valid_bytes_to_u16 () {
    let bytes = [67,218];
    let res = bytes_to_u16(bytes.as_slice());
    assert!(res.is_ok());
    let value: u16 = (256u16.pow(1) * 67) + (256u16.pow(0) * 218);
    assert_eq!(res.unwrap(), value);
}

#[test]
fn invalid_bytes_to_u16 () {
    let bytes = [8,0,1];
    let res = bytes_to_u16(bytes.as_slice());
    assert!(res.is_err());
}

#[test]
fn valid_bytes_to_u64 () {
    let bytes = [1,0,0,0,0,0,5,255];
    let res = bytes_to_u64(bytes.as_slice());
    assert!(res.is_ok());
    let value: u64 = (256u64.pow(7) * 1) + (256u64.pow(1) * 5) + (256u64.pow(0) * 255);
    assert_eq!(res.unwrap(), value);
}

#[test]
fn invalid_bytes_to_u64 () {
    let bytes = [8,0,1];
    let res = bytes_to_u64(bytes.as_slice());
    assert!(res.is_err());
}