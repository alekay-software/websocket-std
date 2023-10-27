use std::time::Instant;
use websocket_std::client::{SyncClient, Config, Reason, WSData, WSEvent};
use websocket_std::result::WebSocketResult;

#[derive(Clone)]
struct Data {
    count: usize,
    connected: bool,
    close: bool,
    start: Instant,
    msg: String
}

type WebSocket<'a> = SyncClient<'a, Data>;

fn websocket_handler(ws: &mut WebSocket, event: WSEvent, data: Option<WSData<Data>>) {
    match event {
        WSEvent::ON_CONNECT => on_connect(ws, data),
        WSEvent::ON_TEXT(msg) => on_message(ws, msg, data),
        WSEvent::ON_CLOSE(reason) => on_close(reason, data)
    }
}

fn on_message(ws: &mut WebSocket, _msg: String, data: Option<WSData<Data>>) {
    let mut data = data.unwrap();
    let mut data = data.borrow_mut();
    data.count += 1;
    println!("[SERVER]: {}", _msg);
    data.msg = _msg;
    ws.send("Hello world").unwrap();
}

fn on_connect(ws: &mut WebSocket, data: Option<WSData<Data>>) {
    println!("Connected");
    let protocol = if ws.protocol().is_some() { ws.protocol().unwrap() } else { "--" };
    println!("Accepted protocol: {protocol}");
    let mut data = data.unwrap();
    let mut data = data.borrow_mut();
    data.connected = true;
    ws.send("Hello world").unwrap();
}

fn on_close(reason: Reason, data: Option<WSData<Data>>) {
    let mut _who_closed = "";
    let mut _code = 0u16;

    match reason {
        Reason::SERVER_CLOSE(c) => {
            _who_closed = "server";
            _code = c;
        },

        Reason::CLIENT_CLOSE(c) => {
            _who_closed = "client";
            _code = c;
        }
    }

    println!("Connection closed by {_who_closed}, code: {_code}");

    let mut data = data.unwrap();
    let mut data = data.borrow_mut();
    data.close = true;
}

fn main() -> WebSocketResult<()> {
    let host: &str = "localhost";
    let port: u16 = 3000;
    let path: &str = "/";

    let data = Data { close: false, count: 0, connected: false, start: Instant::now(), msg: String::new() };
    let mut data: WSData<Data> = WSData::new(data);

    let config = Config {
        callback: Some(websocket_handler), 
        data: Some(data.clone()),
        protocols: Some(&["chat", "superchat"])
    };
    
    println!(
        "Connecting to {host}:{port}{path}",
        host = host,
        port = port,
        path = path
    );

    // List of protocols to accept
    let mut c1 = SyncClient::new();


    if let Some(protocol) = c1.protocol() {
        println!("Accepted protocol: {}", protocol); 
    }

    c1.set_message_size(1024);
    c1.send("Message before the init handshake")?;
    c1.init(host, port, path, Some(config))?;

    let start = Instant::now();
    data.borrow_mut().start = start;
    while !data.borrow().close {
        c1.event_loop()?; 
        if start.elapsed().as_secs() >= 1 { break }
    }

    println!("Connection time {} seconds", start.elapsed().as_secs());

    drop(c1); 
    println!("Finishing connection");
    println!("Last message received: '{}'", data.borrow().msg);
    let count = data.borrow().count;
    println!("Total messages received: {}", count);
    Ok(())
}