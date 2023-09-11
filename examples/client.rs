use websocket_std::result::WebSocketResult;
use websocket_std::client::{sync_connect, SyncClient};
use std::time::Instant;
use std::thread;

fn on_message(msg: String) {
    println!("[CLIENT 1]: {}", msg);
}

fn main() -> WebSocketResult<()> {
    let host: &'static str = "129.151.233.192"; // Make static lifetime, &str lives for the entire lifetime of the running program.
    let port: u16 = 3000;
    let path: &'static str = "/";

    println!("Connecting to {host}:{port}{path}", host = host, port = port, path = path);
    
    let mut client: SyncClient<'static> = sync_connect(host, port, path)?;

    println!("Connected to VAM Scoreboard");

    client.set_response_cb(on_message);

    client.set_message_size(1024);

    let start = Instant::now();

    let t = thread::spawn(move || {
        loop {
            client.event_loop().unwrap();
            if start.elapsed().as_secs() >= 60 { break }            
        }
    });

    t.join().unwrap();
    
    println!("Terminanting main");
    Ok(())
}
