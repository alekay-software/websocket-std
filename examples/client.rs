use std::time::Instant;
use websocket_std::client::{sync_connect, SyncClient};
use websocket_std::result::WebSocketResult;
use std::thread;
use std::sync::Mutex;

struct Data {
    count: Mutex<usize>,
    msg: String,
}

// Raw pointer       = 258620
// Smart Pointer Arc = 

type WebSocket<'a> = SyncClient<'a, Data>;

unsafe fn on_message(ws: &mut WebSocket, msg: String, data: *mut Data) {
    let mut count = (*data).count.lock().unwrap();
    *count += 1;
    (*data).msg = msg;
    let res = ws.send_message("Hello world");
}

fn main() -> WebSocketResult<()> {
    let host: &str = "localhost"; // Make static lifetime, &str lives for the entire lifetime of the running program.
    let port: u16 = 3000;
    let path: &str = "/";
    let data = Data { count: Mutex::new(0), msg: String::new() };
    let data_box = Box::new(data);
    let data = Box::into_raw(data_box);

    println!(
        "Connecting to {host}:{port}{path}",
        host = host,
        port = port,
        path = path
    );

    let mut c1: WebSocket = sync_connect(host, port, path)?;
    let mut c2: WebSocket = sync_connect(host, port, path)?;

    println!("Connected to VAM Scoreboard");

    c1.set_response_cb(on_message, data);
    c1.set_message_size(1024);

    c2.set_response_cb(on_message, data);
    c2.set_message_size(1024);

    let h1 = thread::spawn(move|| {
        c1.send_message("Hello world");
        let start = Instant::now();
        while c1.is_connected() {        
            c1.event_loop();
            if start.elapsed().as_secs() >= 3 { break }            
        }
    });


    let h2 = thread::spawn(move|| {
        c2.send_message("Hello world");
        let start = Instant::now();
        while c2.is_connected() {        
            c2.event_loop();
            if start.elapsed().as_secs() >= 3 { break }            
        }
    });

    h1.join();
    h2.join();
    
    println!("Finishing connection");
    let count = unsafe { *((*data).count.lock().unwrap()) };
    println!("Total messages received: {}", count);
    Ok(())
}
