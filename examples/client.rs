use std::thread;
use std::time::{Duration, Instant};
use websocket_std::client::{sync_connect, SyncClient};
use std::time::{Instant, Duration};
use std::thread;
use std::ptr;

struct Data {
    count: usize,
}

unsafe fn on_message(msg: String, data: *mut Data) {
    (*data).count += 1;
}

fn main() -> WebSocketResult<()> {
    let host: &'static str = "192.168.1.141"; // Make static lifetime, &str lives for the entire lifetime of the running program.
    let port: u16 = 3000;
    let path: &'static str = "/";
    let data_box = Box::new(Data { count: 0 });
    let data = Box::into_raw(data_box);

    println!(
        "Connecting to {host}:{port}{path}",
        host = host,
        port = port,
        path = path
    );

    let mut client: SyncClient<'static, Data> = sync_connect(host, port, path)?;
    println!("Connected to VAM Scoreboard");

    client.set_response_cb(on_message, data);
    client.set_message_size(1024);

    let start = Instant::now();

    let mut i = 1;
    loop {
        if !client.is_connected() { 
            println!("Disconnected");
            break;
        }
        client.event_loop()?;
        thread::sleep(Duration::from_secs(1));
        client.send_message("Hello world")?;
        if start.elapsed().as_secs() >= 20 { break }            
    }
    
    println!("Terminanting main");
    Ok(())
}
