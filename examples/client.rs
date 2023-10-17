use std::time::Instant;
use websocket_std::client::{sync_connect, SyncClient};
use websocket_std::result::WebSocketResult;
use std::sync::Arc;

struct Data {
    count: usize,
    msg: String
}

// Raw pointer       = 258620
// Smart Pointer Arc = 

type WebSocket<'a> = SyncClient<'a, Data>;

unsafe fn on_message(ws: &mut WebSocket, msg: String, data: *mut Data) {
    (*data).count += 1;
    (*data).msg = msg;
    let res = ws.send_message("Hello world");
}

fn main() -> WebSocketResult<()> {
    let host: &str = "localhost"; // Make static lifetime, &str lives for the entire lifetime of the running program.
    let port: u16 = 3000;
    let path: &str = "/";
    let data = Data { count: 0, msg: String::new() };
    let data_box = Box::new(data);
    let data = Box::into_raw(data_box);
    // let data = &mut Arc::new(data);

    println!(
        "Connecting to {host}:{port}{path}",
        host = host,
        port = port,
        path = path
    );

    let mut client: WebSocket = sync_connect(host, port, path)?;

    println!("Connected to VAM Scoreboard");

    client.set_response_cb(on_message, data);
    client.set_message_size(1024);

    let start = Instant::now();

    loop {
        if !client.is_connected() { 
            println!("Disconnected");
            break;
        }
        client.event_loop()?;
        client.send_message("Hello world")?;
        if start.elapsed().as_secs() >= 15 { break }            
    }
    
    println!("Finishing connection");
    println!("Messages sent: {}", unsafe { (*data).count });
    Ok(())
}
