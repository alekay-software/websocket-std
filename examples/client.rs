use websocket_std::result::WebSocketResult;
use websocket_std::{sync_connect, client::SyncClient};
use std::time::{Duration, Instant};
use std::thread::sleep;

fn on_message(msg: &str) {
    println!("[SERVER RESPONSE]: {}", msg);
}

fn main() -> WebSocketResult<()> {
    let host: &'static str = "127.0.0.1"; // Make static lifetime, &str lives for the entire lifetime of the running program.
    let port: u16 = 3000;
    let path: &'static str = "/";

    let mut client: SyncClient<'static> = sync_connect(host, port, path)?;
    println!("Connecting to {host}:{port}{path}", host = host, port = port, path = path);

    client.set_response_cb(on_message);

    // client.set_message_size(800000);
    let msg = String::from("Hello from websocket-std");
    client.send_message(msg)?;

    // sleep(Duration::from_secs(20));
    let start = Instant::now();
    loop {
        // println!("[MAIN]: Event Loop");
        client.event_loop()?;
        if start.elapsed().as_secs() >= 70 { break }
    }

    Ok(())
}
