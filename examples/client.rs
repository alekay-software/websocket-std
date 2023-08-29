use websocket_std::result::WebSocketResult;
use websocket_std::client::{sync_connect, SyncClient};
use std::time::{Duration, Instant};
use std::thread::sleep;


fn on_message(msg: String) {
    if msg.len() > 100 {
        println!("[SERVER RESPONSE]: Message of length {}", msg.len());        
    } else {
        println!("[SERVER RESPONSE]: {}", msg);
    }
}

fn main() -> WebSocketResult<()> {
    let host: &'static str = "localhost"; // Make static lifetime, &str lives for the entire lifetime of the running program.
    let port: u16 = 3001;
    let path: &'static str = "/";

    let mut client: SyncClient<'static> = sync_connect(host, port, path)?;
    println!("Connecting to {host}:{port}{path}", host = host, port = port, path = path);

    client.set_response_cb(on_message);

    client.set_message_size(1024);

    let msg = String::from("Hello from websocket-std");

    client.send_message(msg)?;

    // sleep(Duration::from_secs(20));
    let start = Instant::now();
    let mut i = 0;
    loop {
        client.event_loop()?;
        if start.elapsed().as_secs() >= 5 { break }
    }

    Ok(())
}
