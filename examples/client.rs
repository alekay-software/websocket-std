use std::time::Instant;
use websocket_std::client::{SyncClient, Config, Reason, WSData, create_ws_data};
use websocket_std::result::WebSocketResult;

struct Data {
    count: usize,
    connected: bool,
    close: bool,
    start: Instant,
}

type WebSocket<'a> = SyncClient<'a, Data>;

fn on_message(_ws: &mut WebSocket, _msg: String, data: Option<WSData<Data>>) {
    let data = data.unwrap();
    let mut data = data.borrow_mut();
    data.count += 1;
    println!("[SERVER]: {}", _msg);
    // ws.send("Hello world").unwrap();
}

fn on_connect(ws: &mut WebSocket, data: Option<WSData<Data>>) {
    println!("Connected");
    let protocol = if ws.protocol().is_some() { ws.protocol().unwrap() } else { "--" };
    println!("Accepted protocol: {protocol}");
    let data = data.unwrap();
    let mut data = data.borrow_mut();
    data.connected = true;
    // ws.send("Hello world").unwrap();
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

    let data = data.unwrap();
    let mut data = data.borrow_mut();
    data.close = true;
}

fn main() -> WebSocketResult<()> {
    let host: &str = "localhost";
    let port: u16 = 3000;
    let path: &str = "/";

    let data = Data { close: false, count: 0, connected: false, start: Instant::now() };
    let data: WSData<Data> = create_ws_data(data);

    let config = Config {
        on_connect: Some(on_connect),
        on_data: Some(on_message),
        on_close: Some(on_close),
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
        if start.elapsed().as_secs() >= 10 { break }
    }

    println!("Connection time {} seconds", start.elapsed().as_secs());

    drop(c1); 
    println!("Finishing connection");
    let count = data.borrow().count;
    println!("Total messages received: {}", count);
    Ok(())
}