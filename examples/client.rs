use std::time::Instant;
use websocket_std::client::{sync_connect, SyncClient};
use websocket_std::result::WebSocketResult;
use std::sync::Arc;
use std::cell::RefCell;
use websocket_std::extension::Parameter;
use websocket_std::parameter;
use std::collections::HashMap;


struct Data {
    count: usize,
}

type WebSocket<'a> = SyncClient<'a, RefCell<Data>>;
type WSData = Arc<RefCell<Data>>;

fn on_message(ws: &mut WebSocket, _msg: String, data: Option<WSData>) {
    let data = data.unwrap();
    let mut data = data.borrow_mut();
    data.count += 1;
    println!("[SERVER]: {}", _msg);
    ws.send("Hello world").unwrap();
}

fn main() -> WebSocketResult<()> {
    let host: &str = "localhost";
    let port: u16 = 3000;
    let path: &str = "/";
    let data: WSData = Arc::new(RefCell::new(Data { count: 0 }));
    
    let p1 = parameter!("person");
    let p2 = parameter!("person"; "name");
    let p3 = parameter!("person"; "name=sergio", "apellido=ramirez ojea");
    let p4 = parameter!("person"; "name=sergio", "apellido=ramirez", "edad=24");
    
    p1.print();
    println!();
    p2.print();
    println!();
    p3.print();
    println!();
    p4.print();
    println!();

    return Ok(());

    println!(
        "Connecting to {host}:{port}{path}",
        host = host,
        port = port,
        path = path
    );

    // List of protocols to accept
    let protocols = ["socoreboard", "chat"];
    let mut c1: WebSocket = sync_connect(host, port, path, Some(&protocols))?;

    if let Some(protocol) = c1.protocol() {
        println!("Accepted protocol: {}", protocol); 
    }

    println!("Connected to VAM Scoreboard");

    c1.set_response_cb(on_message, Some(data.clone()));
    c1.set_message_size(1024);

    c1.send("Hello world")?;
    let start = Instant::now();
    while c1.is_connected() {
        c1.event_loop()?; 
        if start.elapsed().as_secs() >= 40 { break }
    }
    
    println!("Finishing connection");
    let count = data.borrow().count;
    println!("Total messages received: {}", count);
    Ok(())
}