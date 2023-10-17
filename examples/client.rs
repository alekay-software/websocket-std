use std::time::Instant;
use websocket_std::client::{sync_connect, SyncClient};
use websocket_std::result::WebSocketResult;
use std::sync::Arc;
use std::cell::RefCell;

struct Data {
    count: usize,
}

type WebSocket<'a> = SyncClient<'a, RefCell<Data>>;
type WSData = Arc<RefCell<Data>>;

fn on_message(ws: &mut WebSocket, _msg: String, data: Option<WSData>) {
    let data = data.unwrap();
    let mut data = data.borrow_mut();
    data.count += 1;
    ws.send_message("Hello world").unwrap();
}

fn main() -> WebSocketResult<()> {
    let host: &str = "localhost"; // Make static lifetime, &str lives for the entire lifetime of the running program.
    let port: u16 = 3000;
    let path: &str = "/";
    let data: WSData = Arc::new(RefCell::new(Data { count: 0 }));

    println!(
        "Connecting to {host}:{port}{path}",
        host = host,
        port = port,
        path = path
    );

    let mut c1: WebSocket = sync_connect(host, port, path)?;

    println!("Connected to VAM Scoreboard");

    c1.set_response_cb(on_message, Some(data.clone()));
    c1.set_message_size(1024);

    c1.send_message("Hello world")?;
    let start = Instant::now();
    while c1.is_connected() {        
        c1.event_loop()?;
        if start.elapsed().as_secs() >= 60 { break }            
    }
    
    println!("Finishing connection");
    let count = data.borrow().count;
    println!("Total messages received: {}", count);
    Ok(())
}