use websocket_std::result::WebSocketResult;
use websocket_std::client::{sync_connect, SyncClient};
use std::time::{Instant, Duration};
use std::thread::{self, sleep};
use std::ptr;

struct Data {
    name: String,
    count: usize
}

unsafe fn on_message(msg: String, data: *mut Data) {
    (*data).count += 1;
}

fn main() -> WebSocketResult<()> {
    let host: &'static str = "localhost"; // Make static lifetime, &str lives for the entire lifetime of the running program.
    let port: u16 = 3000;
    let path: &'static str = "/";

    let data_box = Box::new(Data { name: String::from("Alejandro"), count: 0 });
    let data = Box::into_raw(data_box);

    println!("Connecting to {host}:{port}{path}", host = host, port = port, path = path);
    
    let mut client: SyncClient<'static, Data> = sync_connect(host, port, path)?;

    println!("Connected to VAM Scoreboard");

    client.set_response_cb(on_message, data);
    client.set_message_size(1024);

    let start = Instant::now();
  
    loop {
        if start.elapsed().as_secs() >= 3 { break }            
        client.send_message("Hello world")?;

        if !client.is_connected() { 
            println!("Disconnected");
            break 
        }
        client.event_loop()?;
    }
    
    println!("Messages sent: {}", unsafe { (*data).count} );
    Ok(())
}
