// use super::net::read_entire_tcp_package;
// use std::net::{TcpStream, TcpListener, Shutdown};
// use std::io::{BufReader, Write};


// // Setup server and client socket
// // Client socket must be in not_blocking mode
// // Test run in parallel so create socket with different ports to avoid errors taken the same address
// fn setup() -> (TcpStream, BufReader<TcpStream>, TcpStream) {
//     // With port 0 the program will request and available random port
//     let listener = TcpListener::bind("localhost:0").unwrap();
//     let port = listener.local_addr().unwrap().port();
//     let client = TcpStream::connect(format!("localhost:{}", port)).unwrap();
//     client.set_nonblocking(true).unwrap();
//     let (server, _) = listener.accept().unwrap();
//     let client_buffer = BufReader::new(client.try_clone().unwrap());

//     (server, client_buffer, client)
// }

// fn before_each(server: TcpStream, client: TcpStream) {
//     server.shutdown(Shutdown::Both);
//     client.shutdown(Shutdown::Both);
// }

// fn wait_until_data_is_receive(reader: &mut BufReader<TcpStream>, data: &mut Vec<u8>) {
//     // Wait until data is received
//     while data.len() <= 0 {
//         let d = read_entire_tcp_package(reader);
//         let d = d.unwrap();
//         if d.len() > 0 {
//             data.extend(d);
//         }
//     }
// }

// #[test]
// fn read_empty() {
//     let (server, mut reader, client) = setup();
//     let data = read_entire_tcp_package(&mut reader);
//     assert!(data.is_ok());
//     assert_eq!(data.unwrap().len(), 0);

//     before_each(server, client);
// }

// #[test]
// fn read_less_than_buffer_capacity() {
//     let (mut server, mut reader, client) = setup();
//     let data_to_send = "hello".as_bytes();
//     assert!(data_to_send.len() < reader.capacity());
//     assert!(server.write_all(data_to_send).is_ok());

//     let mut data = Vec::new();
//     wait_until_data_is_receive(&mut reader, &mut data);

//     assert_eq!(data.len(), data_to_send.len());
//     assert!(data.len() < reader.capacity());

//     before_each(server, client);
// }

// #[test]
// fn read_exact_buffer_capacity() {
//     let (mut server, mut reader, client) = setup();
    
//     let mut data_to_send: Vec<u8> = Vec::new();
//     for _ in 0..reader.capacity() {
//         data_to_send.push('a' as u8);
//     }

//     assert!(data_to_send.len() == reader.capacity());
//     assert!(server.write_all(data_to_send.as_slice()).is_ok());

//     let mut data = Vec::new();
//     wait_until_data_is_receive(&mut reader, &mut data);

//     assert_eq!(data.len(), data_to_send.len());
//     assert!(data.len() == reader.capacity());

//     before_each(server, client);
// }

// #[test]
// fn read_more_than_buffer_capacity() {
//     let (mut server, mut reader, client) = setup();
    
//     let mut data_to_send: Vec<u8> = Vec::new();
//     for _ in 0..reader.capacity() + 10 {
//         data_to_send.push('a' as u8);
//     }

//     assert!(data_to_send.len() > reader.capacity());
//     assert!(server.write_all(data_to_send.as_slice()).is_ok());
    
//     let mut data = Vec::new();
//     wait_until_data_is_receive(&mut reader, &mut data);

//     assert_eq!(data.len(), data_to_send.len());
//     assert!(data.len() > reader.capacity());

//     before_each(server, client);

// }